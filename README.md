# pylon-cpp-rs

English | [简体中文](README.zh-CN.md)

Safe Rust bindings for the Basler pylon C++ SDK (pylon 12), built on a thin C shim over the C++ API.

## Status

| Layer | Path | State |
|---|---|---|
| C shim | `pylon-sys/shim` | done |
| Raw bindings (`-sys`, bindgen) | `pylon-sys` | done; verified on Windows, untested on Linux and macOS |
| Safe Rust API | crate root | done; verified on Windows with the camera emulator, untested on Linux, macOS and real cameras |

Scope: camera control and basic pixel/format conversion. GUI functions are out of scope.

Platforms: Windows x64, Linux x86_64 / aarch64, macOS x86_64 / arm64.

## Layout

```text
src/lib.rs                      safe Rust API: runtime, errors, crate documentation
src/camera.rs                   device infos, cameras, node callbacks, grab results
src/node.rs                     node maps and nodes
src/image.rs                    pixel types, image views, images, format converter
pylon-sys/shim/pylon_shim.h     C header, the only bindgen input
pylon-sys/shim/pylon_shim.cpp   C++17 implementation on top of the pylon C++ SDK
pylon-sys/src/bindings.rs       bindgen output, committed; the same for every platform
pylon-sys/build.rs              compiles the shim and links pylon
```

## Requirements

- pylon 12 SDK and a C++17 compiler. `pylon-sys/build.rs` locates the SDK as follows:

| Platform | SDK location | Compiler and linker settings | pylon libraries at run time |
|---|---|---|---|
| Windows | `PYLON_DEV_DIR`, set by the installer | `<PYLON_DEV_DIR>/include`, `<PYLON_DEV_DIR>/lib/x64`; the pylon headers select their libraries through `#pragma comment(lib)` | the installer puts the runtime DLLs on `PATH` |
| Linux | `PYLON_ROOT`, default `/opt/pylon` | `<PYLON_ROOT>/bin/pylon-config --cflags` / `--libs` | `LD_LIBRARY_PATH`, or the output of `pylon-config --libs-rpath` as link arguments of the final binary |
| macOS | `PYLON_ROOT`, default `/Library/Frameworks/pylon.framework` | framework search path is the parent of `PYLON_ROOT`, plus `<PYLON_ROOT>/Headers/GenICam` | runpath of the final binary, e.g. `cargo::rustc-link-arg=-Wl,-rpath,/Library/Frameworks` in its build script |

- Cargo passes link arguments only to the binaries of the package that emits them, so `pylon-sys` cannot set the runpath of downstream binaries.
- Tests without hardware can use the pylon camera emulator: set `PYLON_CAMEMU=<count>`.

## Example

```rust
use pylon_cpp_rs::{Camera, Converter, GrabStrategy, Image, PixelType, Pylon};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pylon = Pylon::new()?;
    let camera = Camera::new(&pylon, None)?; // first device found
    camera.open()?;
    let width = camera.node_map()?.node("Width")?.ok_or("no Width node")?.integer()?;
    println!("width {width}");

    let mut converter = Converter::new(&pylon)?;
    converter.set_output_pixel_type(PixelType::BGR8_PACKED)?;
    let mut image = Image::new(&pylon)?;
    camera.start_grabbing(GrabStrategy::OneByOne, 10)?; // stops after 10 results
    while let Some(result) = camera.retrieve_result(5000)? {
        let info = result.info()?;
        let Some(source) = info.image else {
            eprintln!("grab failed: {}", result.error_description()?);
            continue;
        };
        converter.convert(&mut image, &source)?;
        let bgr = image.view().ok_or("empty image")?;
        println!("frame {}: {}x{}, {} bytes", info.block_id, bgr.width(), bgr.height(), bgr.data().len());
    }
    Ok(())
}
```

## Regenerating the bindings

After changing `pylon_shim.h`, run:

```bash
cargo build -p pylon-sys --features bindgen
```

This needs LLVM: `libclang` (found through `LIBCLANG_PATH` or `PATH`) and the `clang` executable on `PATH`, which bindgen uses to locate the compiler's own headers. The build script generates the bindings for all five supported targets with `-ffreestanding` and fails if they differ, so the single committed file fits every platform.

## C shim conventions

The comment at the top of `pylon_shim.h` is the reference for handles, errors, strings, callbacks and threads; the design in short:

- Every C++ exception becomes a `PylonStatus` named after its GenICam exception class.
- Strings and lists are delivered through synchronous callbacks.
- The camera opens only through `pylon_camera_open`, `pylon_camera_start_grabbing` and `pylon_camera_grab_one`, and closes only through `pylon_camera_close` and `pylon_camera_destroy`, so node maps never become invalid behind the caller's back.
- Handlers that pylon calls from its own threads are left out. Grabbing runs in the thread that calls `pylon_camera_retrieve_result`, and camera events arrive as node callbacks inside that call or inside `pylon_camera_grab_one`.
- A node callback registration owns its context and releases it through a drop callback when the node map ends; registrations cannot be removed.
- pylon enums and all `EPixelType` values are mirrored and checked with `static_assert`.
- Every shim enum has `int32_t` as fixed underlying type (C23, C++11), so its ABI is the same on every platform; bindgen turns it into a newtype such as `PylonStatus(pub i32)` that also holds values outside the listed constants.

## Rust API conventions

The crate documentation (`cargo doc --open`) describes the runtime, thread, borrowing and callback rules. The pylon features that the safe API rejects are listed, with the reasons, under Not wrapped.

## API mapping

### Runtime and errors

| pylon C++ API | C shim | Rust API |
|---|---|---|
| `PylonInitialize` / `PylonTerminate` | `pylon_initialize` / `pylon_terminate` | `Pylon::new` / drop of the last `Pylon` (one runtime at a time; clone for more references) |
| `GetPylonVersion` | `pylon_version` | `version` |
| `GenericException` and subclasses | `PylonStatus`, `pylon_last_error` | `Error` (`kind`, `message`), `ErrorKind`, `Result` |

### Devices

| pylon C++ API | C shim | Rust API |
|---|---|---|
| `CTlFactory::EnumerateDevices` (with and without filter) | `pylon_enumerate_devices` | `Pylon::enumerate` |
| `CDeviceInfo()` | `pylon_device_info_create` / `pylon_device_info_destroy` | `DeviceInfo::new` / drop |
| `CDeviceInfo::GetPropertyValue`, `Get<Key>` accessors | `pylon_device_info_get` | `DeviceInfo::get` |
| `CDeviceInfo::SetPropertyValue`, `Set<Key>` accessors | `pylon_device_info_set` | `DeviceInfo::set` |
| `CDeviceInfo::GetPropertyNames` | `pylon_device_info_keys` | `DeviceInfo::keys` |

### Camera

| pylon C++ API | C shim | Rust API |
|---|---|---|
| `CTlFactory::CreateDevice` / `CreateFirstDevice` + `CInstantCamera(IPylonDevice*)` | `pylon_camera_create` | `Camera::new` |
| `CInstantCamera::~CInstantCamera` | `pylon_camera_destroy` | drop `Camera` |
| `GetDeviceInfo` | `pylon_camera_device_info` | `Camera::device_info` |
| `Open` / `Close` / `IsOpen` | `pylon_camera_open` / `pylon_camera_close` / `pylon_camera_is_open` | `Camera::open` / drop `Camera` / `Camera::is_open` |
| `IsCameraDeviceRemoved` | `pylon_camera_is_device_removed` | `Camera::is_device_removed` |
| `GetNodeMap` / `GetTLNodeMap` / `GetStreamGrabberNodeMap` / `GetEventGrabberNodeMap` / `GetInstantCameraNodeMap` | `pylon_camera_node_map` | `Camera::node_map` / `tl_node_map` / `stream_grabber_node_map` / `event_grabber_node_map` / `instant_camera_node_map` |
| `RegisterConfiguration` with `CAcquireContinuousConfiguration` / `CAcquireSingleFrameConfiguration` / `CSoftwareTriggerConfiguration` | `pylon_camera_set_configuration` | `Camera::set_configuration`, `Configuration` |
| `StartGrabbing` (both overloads) | `pylon_camera_start_grabbing` | `Camera::start_grabbing`, `GrabStrategy` |
| `StopGrabbing` / `IsGrabbing` | `pylon_camera_stop_grabbing` / `pylon_camera_is_grabbing` | `Camera::stop_grabbing` / `Camera::is_grabbing` |
| `RetrieveResult` | `pylon_camera_retrieve_result` | `Camera::retrieve_result` |
| `GrabOne` | `pylon_camera_grab_one` | `Camera::grab_one` |
| `CanWaitForFrameTriggerReady` / `WaitForFrameTriggerReady` | `pylon_camera_can_wait_for_frame_trigger_ready` / `pylon_camera_wait_for_frame_trigger_ready` | `Camera::can_wait_for_frame_trigger_ready` / `Camera::wait_for_frame_trigger_ready` |
| `ExecuteSoftwareTrigger` | `pylon_camera_execute_software_trigger` | `Camera::execute_software_trigger` |
| Instant camera parameters (`MaxNumBuffer`, `OutputQueueSize`, `GrabCameraEvents`, …) | node API on `PYLON_NODE_MAP_INSTANT_CAMERA` | `Camera::instant_camera_node_map` + `Node` |

### Grab result

| pylon C++ API | C shim | Rust API |
|---|---|---|
| `CGrabResultPtr` copy / release | `PylonGrabResult` / `pylon_grab_result_destroy` | `GrabResult` / drop |
| `CGrabResultData` getters and `operator IImage&` | `pylon_grab_result_info` | `GrabResult::info`, `GrabResultInfo`, `PayloadType` |
| `GetErrorDescription` | `pylon_grab_result_error_description` | `GrabResult::error_description` |
| `GetChunkDataNodeMap` | `pylon_grab_result_chunk_node_map` | `GrabResult::chunk_node_map` |

### Node maps and nodes

| pylon C++ API | C shim | Rust API |
|---|---|---|
| `INodeMap::GetNode` | `pylon_node_map_node` | `NodeMap::node` |
| `CFeaturePersistence::LoadFromString` / `SaveToString` | `pylon_node_map_load` / `pylon_node_map_save` | `NodeMap::load` / `NodeMap::save` |
| `INode::GetPrincipalInterfaceType` | `pylon_node_type` | `Node::node_type`, `NodeType` |
| `IBase::GetAccessMode` (`IsReadable` / `IsWritable` / `IsAvailable`) | `pylon_node_access_mode` | `Node::access_mode`, `AccessMode::is_readable` / `is_writable` / `is_available` |
| `INode::GetName` / `GetDisplayName` / `GetToolTip` / `GetDescription` | `pylon_node_text` | `Node::name` / `display_name` / `tool_tip` / `description` |
| `GenApi::Register`, `CCameraEventHandler` | `pylon_node_register_callback` | `Camera::register_callback` |
| `IValue::ToString` / `FromString` (also enumeration and string values) | `pylon_value_to_string` / `pylon_value_from_string` | `Node::value` / `Node::set_value` |
| `IInteger::GetValue` / `SetValue` / `GetMin` / `GetMax` / `GetInc` | `pylon_integer_get` / `pylon_integer_set` / `pylon_integer_range` | `Node::integer` / `set_integer` / `integer_range` |
| `IFloat::GetValue` / `SetValue` / `GetMin` / `GetMax` / `GetInc` | `pylon_float_get` / `pylon_float_set` / `pylon_float_range` | `Node::float` / `set_float` / `float_range` |
| `IBoolean::GetValue` / `SetValue` | `pylon_boolean_get` / `pylon_boolean_set` | `Node::boolean` / `set_boolean` |
| `ICommand::Execute` / `IsDone` | `pylon_command_execute` / `pylon_command_is_done` | `Node::execute` / `is_done` |
| `IEnumeration::GetEntries` | `pylon_enumeration_entries` | `Node::entries` |
| `IEnumEntry::GetSymbolic` | `pylon_enum_entry_symbolic` | `Node::symbolic` |
| `ICategory::GetFeatures` | `pylon_category_features` | `Node::features` |

### Pixel types, images and conversion

| pylon C++ API | C shim | Rust API |
|---|---|---|
| `EPixelType` | `PYLON_PIXEL_TYPE_*` | `PixelType::*` |
| `BitPerPixel`, `BitDepth`, `SamplesPerPixel`, `PlaneCount`, `IsMonoImage` (`IsColorImage` is its complement), `IsBayer`, `IsPacked`, `HasAlpha`, `CPixelTypeMapper::GetNameByPixelType` | `pylon_pixel_type_info` | `PixelType::info`, `PixelTypeInfo` |
| `CPixelTypeMapper::GetPylonPixelTypeByName` | `pylon_pixel_type_from_name` | `PixelType::from_name` |
| `ComputeStride` | `pylon_pixel_type_stride` | `PixelType::stride` |
| `CPylonImage()` and its `IImage` getters | `pylon_image_create` / `pylon_image_destroy` / `pylon_image_view` | `Image::new` / drop / `Image::view`, `ImageView`, `Orientation` |
| `CImageFormatConverter()` | `pylon_converter_create` / `pylon_converter_destroy` | `Converter::new` / drop |
| `CImageFormatConverter::GetNodeMap` (`Gamma`, `OutputBitAlignment`, `OutputOrientation`, …) | `pylon_converter_node_map` | `Converter::node_map` |
| `CImageFormatConverter::OutputPixelFormat` | `pylon_converter_set_output_pixel_type` | `Converter::set_output_pixel_type` |
| `CImageFormatConverter::ImageHasDestinationFormat` | `pylon_converter_has_destination_format` | `Converter::has_destination_format` |
| `CImageFormatConverter::Convert` | `pylon_converter_convert` | `Converter::convert`; sources from `GrabResultInfo::image`, `Image::view` or `ImageView::new` |

## Not wrapped

| pylon C++ API | Status | Reason |
|---|---|---|
| GenDC multi-component data: `CPylonDataContainer`, `CPylonDataComponent`, `GetDataComponent*` | not yet | only the first image component of a GenDC payload is exposed, through the grab result image |
| Compression Beyond: `CImageDecompressor` | not yet | |
| Explicit event grabbing: `StartEventGrabbing`, `ProcessOneEvent`, `StopEventGrabbing`, `IsEventGrabbing`, `GetCameraEventWaitObject` | not yet | events during grabbing are delivered through `GrabCameraEvents` |
| GigE specific: action commands, IP configuration, multicast | not yet | |
| Further GenApi members: `INodeMap::GetNodes` / `Poll` / `InvalidateNodes` / `GetLock`, `IRegister`, `IPort`, units, visibility, representation, list increments | not yet | without `GetLock`, sequences such as selector plus value are not atomic across threads |
| Copies of `CDeviceInfo` / `CGrabResultPtr` (Rust `Clone`) | not yet | the shim has no copy function; enumerate again or call `Camera::device_info` |
| `InconvertibleEdgeHandling` `Clip` / `Extend` of the converter | rejected | pylon writes past planar destinations with `Clip` and reads before 16-bit Bayer sources with `Extend`; `Converter::convert` fails unless it is `SetZero` |
| Converter outputs `PixelType_YUV422planar` / `PixelType_YUV420planar` | rejected | pylon leaves some chroma bytes of these outputs unwritten; `Converter::convert` fails for them |
| `GenApi::Deregister` | no need | removed from the shim: it neither waits for running callbacks nor tolerates concurrent ones, and its handle is the callback object, so a second call is a use after free; registrations end with the camera, and a flag checked in the closure stops reacting |
| Node callbacks on other node maps (transport layer, stream and event grabber, instant camera, chunk data, converter) | no need | chunk node maps are reused with their buffers; frame grabber transport layer events come from internal threads; camera event data nodes live in the camera node map |
| PylonGUI (`CPylonImageWindow`, `DisplayImage`), `CPylonBitmapImage` | no need | GUI is out of scope |
| `CImagePersistence`, `CPylonImage::Save` / `Load`, `CAviWriter`, `CVideoWriter` | no need | image files are handled on the Rust side |
| Grab loop thread (`GrabLoop_ProvidedByInstantCamera`), `CImageEventHandler` | no need | a Rust thread calling `Camera::retrieve_result` does the same in a thread the caller owns |
| Custom `CConfigurationEventHandler` subclasses, device removal callback | no need | `Camera::is_device_removed` after a failed call, as in the pylon DeviceRemovalHandling sample |
| `Attach` / `DetachDevice` / `DestroyDevice` / `HasOwnership` | no need | a camera owns its device for its whole life; reconnecting creates a new camera |
| Low level API: `IPylonDevice`, `IStreamGrabber`, `IEventGrabber`, `ITransportLayer`, `CTlFactory::CreateTl` / `EnumerateTls`, `CInstantInterface` | no need | covered by the instant camera |
| `CInstantCameraArray` | no need | several cameras are coordinated in Rust |
| `IBufferFactory`, `SetCameraContext` / `GetCameraContext`, `GetQueuedBufferCount` (deprecated) | no need | advanced or deprecated; queue counts are nodes of the instant camera node map |
| `GetSfncVersion`, `IsGigE` / `IsUsb` / `IsCameraLink` / `IsCxp`, `IsDeviceAccessible` | no need | available from nodes, from device info keys, or from the result of opening |
| `C*Parameter` helpers: value correction, `TrySet*`, `*OrDefault`, percent of range | no need | composed in Rust from range and access mode |
| `CFeaturePersistence::Load` / `Save` (files) | no need | string variants plus file I/O in Rust |
| `CPylonImage` extras: `CopyImage`, `AttachUserBuffer`, `AttachGrabResultBuffer`, `GetAoi`, `GetPlane`, `ChangePixelType` | no need | Rust slices cover buffer views and copies; owned per-frame results come from reusing several `Image` objects |
| `CImageFormatConverter` extras: `Initialize`, `GetBufferSizeForConversion`, `IsSupportedInputFormat` / `IsSupportedOutputFormat`, raw destination `Convert` overloads | no need | converting into a reused `Image` covers them; unsupported formats are reported as errors |
