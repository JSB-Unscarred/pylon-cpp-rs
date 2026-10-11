# pylon-cpp-rs

[English](README.md) | 简体中文

Basler pylon C++ SDK（pylon 12）的 Rust 安全包装，底层是一层很薄的 C shim。

## 进度

| 层 | 路径 | 状态 |
|---|---|---|
| C shim | `pylon-sys/shim` | 已完成 |
| 原始绑定（`-sys`，bindgen） | `pylon-sys` | 已完成；已在 Windows 验证，Linux 与 macOS 尚未测试 |
| Rust 安全 API | crate 根目录 | 已完成；已在 Windows 上用相机模拟器验证，Linux、macOS 与真实相机尚未测试 |

范围：相机操作与基础的像素、格式转换。GUI 相关功能不在范围内。

平台：Windows x64、Linux x86_64 / aarch64、macOS x86_64 / arm64。

## 目录

```text
src/lib.rs                      Rust 安全 API：运行时、错误、crate 文档
src/camera.rs                   设备信息、相机、节点回调、grab result
src/node.rs                     node map 与节点
src/image.rs                    像素类型、图像视图、图像、格式转换
pylon-sys/shim/pylon_shim.h     C 头文件，bindgen 唯一输入
pylon-sys/shim/pylon_shim.cpp   基于 pylon C++ SDK 的 C++17 实现
pylon-sys/src/bindings.rs       bindgen 输出，已入库，各平台通用
pylon-sys/build.rs              编译 shim 并链接 pylon
```

## 环境要求

- pylon 12 SDK 与 C++17 编译器。`pylon-sys/build.rs` 按下表定位 SDK：

| 平台 | SDK 位置 | 编译与链接设置 | 运行时查找 pylon 库 |
|---|---|---|---|
| Windows | `PYLON_DEV_DIR`，由安装程序设置 | `<PYLON_DEV_DIR>/include`、`<PYLON_DEV_DIR>/lib/x64`；pylon 头文件通过 `#pragma comment(lib)` 选择库 | 安装程序把运行时 DLL 目录加入 `PATH` |
| Linux | `PYLON_ROOT`，默认 `/opt/pylon` | `<PYLON_ROOT>/bin/pylon-config --cflags` / `--libs` | `LD_LIBRARY_PATH`，或把 `pylon-config --libs-rpath` 的输出作为最终二进制的链接参数 |
| macOS | `PYLON_ROOT`，默认 `/Library/Frameworks/pylon.framework` | framework 搜索路径为 `PYLON_ROOT` 的父目录，另加 `<PYLON_ROOT>/Headers/GenICam` | 最终二进制的 runpath，例如在其 build script 中输出 `cargo::rustc-link-arg=-Wl,-rpath,/Library/Frameworks` |

- Cargo 只把链接参数传给输出它的 package 自己的二进制，所以 `pylon-sys` 无法设置下游二进制的 runpath。
- 无硬件测试可使用 pylon 相机模拟器：设置 `PYLON_CAMEMU=<数量>`。

## 示例

```rust
use pylon_cpp_rs::{Camera, Converter, GrabStrategy, Image, PixelType, Pylon};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pylon = Pylon::new()?;
    let camera = Camera::new(&pylon, None)?; // 找到的第一台设备
    camera.open()?;
    let width = camera.node_map()?.node("Width")?.ok_or("no Width node")?.integer()?;
    println!("width {width}");

    let mut converter = Converter::new(&pylon)?;
    converter.set_output_pixel_type(PixelType::BGR8_PACKED)?;
    let mut image = Image::new(&pylon)?;
    camera.start_grabbing(GrabStrategy::OneByOne, 10)?; // 取到 10 个结果后停止
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

## 重新生成绑定

修改 `pylon_shim.h` 后运行：

```bash
cargo build -p pylon-sys --features bindgen
```

需要 LLVM：`libclang`（经 `LIBCLANG_PATH` 或 `PATH` 查找）以及 `PATH` 中的 `clang` 可执行文件，bindgen 用它定位编译器自带的头文件。build script 以 `-ffreestanding` 为五个支持的 target 分别生成绑定，结果不一致时报错，因此入库的单个文件适用于所有平台。

## C shim 约定

handle、错误、字符串、回调与线程的约定以 `pylon_shim.h` 顶部注释为准，设计要点如下：

- 每个 C++ 异常转换为以其 GenICam 异常类命名的 `PylonStatus`。
- 字符串和列表经同步回调交付。
- 相机只经 `pylon_camera_open`、`pylon_camera_start_grabbing`、`pylon_camera_grab_one` 打开，只经 `pylon_camera_close`、`pylon_camera_destroy` 关闭，node map 不会在调用方不知情时失效。
- pylon 在其内部线程调用的 handler 一律不包装。采集在调用 `pylon_camera_retrieve_result` 的线程中进行，相机事件以节点回调的形式在该调用或 `pylon_camera_grab_one` 内触发。
- 节点回调的注册接管其上下文，node map 结束时经 drop 回调释放；注册不能撤销。
- pylon 枚举和全部 `EPixelType` 值在头文件中镜像，并用 `static_assert` 校验。
- shim 的每个 enum 都以 `int32_t` 为固定底层类型（C23、C++11），ABI 在各平台一致；bindgen 将其生成为 `PylonStatus(pub i32)` 这样的 newtype，也能容纳列出常量之外的值。

## Rust API 约定

运行时、线程、借用与回调的规则见 crate 文档（`cargo doc --open`）。安全 API 拒绝的 pylon 功能及原因列在“暂不包装 / 无需包装”表中。

## API 对照

### 运行时与错误

| pylon C++ API | C shim | Rust API |
|---|---|---|
| `PylonInitialize` / `PylonTerminate` | `pylon_initialize` / `pylon_terminate` | `Pylon::new` / 最后一个 `Pylon` 的 drop（同一时刻只有一个运行时；更多引用用 clone） |
| `GetPylonVersion` | `pylon_version` | `version` |
| `GenericException` 及其子类 | `PylonStatus`、`pylon_last_error` | `Error`（`kind`、`message`）、`ErrorKind`、`Result` |

### 设备

| pylon C++ API | C shim | Rust API |
|---|---|---|
| `CTlFactory::EnumerateDevices`（带/不带过滤） | `pylon_enumerate_devices` | `Pylon::enumerate` |
| `CDeviceInfo()` | `pylon_device_info_create` / `pylon_device_info_destroy` | `DeviceInfo::new` / drop |
| `CDeviceInfo::GetPropertyValue`、各 `Get<Key>` | `pylon_device_info_get` | `DeviceInfo::get` |
| `CDeviceInfo::SetPropertyValue`、各 `Set<Key>` | `pylon_device_info_set` | `DeviceInfo::set` |
| `CDeviceInfo::GetPropertyNames` | `pylon_device_info_keys` | `DeviceInfo::keys` |

### 相机

| pylon C++ API | C shim | Rust API |
|---|---|---|
| `CTlFactory::CreateDevice` / `CreateFirstDevice` + `CInstantCamera(IPylonDevice*)` | `pylon_camera_create` | `Camera::new` |
| `CInstantCamera::~CInstantCamera` | `pylon_camera_destroy` | drop `Camera` |
| `GetDeviceInfo` | `pylon_camera_device_info` | `Camera::device_info` |
| `Open` / `Close` / `IsOpen` | `pylon_camera_open` / `pylon_camera_close` / `pylon_camera_is_open` | `Camera::open` / drop `Camera` / `Camera::is_open` |
| `IsCameraDeviceRemoved` | `pylon_camera_is_device_removed` | `Camera::is_device_removed` |
| `GetNodeMap` / `GetTLNodeMap` / `GetStreamGrabberNodeMap` / `GetEventGrabberNodeMap` / `GetInstantCameraNodeMap` | `pylon_camera_node_map` | `Camera::node_map` / `tl_node_map` / `stream_grabber_node_map` / `event_grabber_node_map` / `instant_camera_node_map` |
| `RegisterConfiguration` 配合 `CAcquireContinuousConfiguration` / `CAcquireSingleFrameConfiguration` / `CSoftwareTriggerConfiguration` | `pylon_camera_set_configuration` | `Camera::set_configuration`、`Configuration` |
| `StartGrabbing`（两个重载） | `pylon_camera_start_grabbing` | `Camera::start_grabbing`、`GrabStrategy` |
| `StopGrabbing` / `IsGrabbing` | `pylon_camera_stop_grabbing` / `pylon_camera_is_grabbing` | `Camera::stop_grabbing` / `Camera::is_grabbing` |
| `RetrieveResult` | `pylon_camera_retrieve_result` | `Camera::retrieve_result` |
| `GrabOne` | `pylon_camera_grab_one` | `Camera::grab_one` |
| `CanWaitForFrameTriggerReady` / `WaitForFrameTriggerReady` | `pylon_camera_can_wait_for_frame_trigger_ready` / `pylon_camera_wait_for_frame_trigger_ready` | `Camera::can_wait_for_frame_trigger_ready` / `Camera::wait_for_frame_trigger_ready` |
| `ExecuteSoftwareTrigger` | `pylon_camera_execute_software_trigger` | `Camera::execute_software_trigger` |
| InstantCamera 参数（`MaxNumBuffer`、`OutputQueueSize`、`GrabCameraEvents` 等） | `PYLON_NODE_MAP_INSTANT_CAMERA` 上的节点 API | `Camera::instant_camera_node_map` + `Node` |

### Grab result

| pylon C++ API | C shim | Rust API |
|---|---|---|
| `CGrabResultPtr` 复制 / 释放 | `PylonGrabResult` / `pylon_grab_result_destroy` | `GrabResult` / drop |
| `CGrabResultData` 各 getter 与 `operator IImage&` | `pylon_grab_result_info` | `GrabResult::info`、`GrabResultInfo`、`PayloadType` |
| `GetErrorDescription` | `pylon_grab_result_error_description` | `GrabResult::error_description` |
| `GetChunkDataNodeMap` | `pylon_grab_result_chunk_node_map` | `GrabResult::chunk_node_map` |

### Node map 与节点

| pylon C++ API | C shim | Rust API |
|---|---|---|
| `INodeMap::GetNode` | `pylon_node_map_node` | `NodeMap::node` |
| `CFeaturePersistence::LoadFromString` / `SaveToString` | `pylon_node_map_load` / `pylon_node_map_save` | `NodeMap::load` / `NodeMap::save` |
| `INode::GetPrincipalInterfaceType` | `pylon_node_type` | `Node::node_type`、`NodeType` |
| `IBase::GetAccessMode`（`IsReadable` / `IsWritable` / `IsAvailable`） | `pylon_node_access_mode` | `Node::access_mode`、`AccessMode::is_readable` / `is_writable` / `is_available` |
| `INode::GetName` / `GetDisplayName` / `GetToolTip` / `GetDescription` | `pylon_node_text` | `Node::name` / `display_name` / `tool_tip` / `description` |
| `GenApi::Register`、`CCameraEventHandler` | `pylon_node_register_callback` | `Camera::register_callback` |
| `IValue::ToString` / `FromString`（含枚举与字符串的取值、设值） | `pylon_value_to_string` / `pylon_value_from_string` | `Node::value` / `Node::set_value` |
| `IInteger::GetValue` / `SetValue` / `GetMin` / `GetMax` / `GetInc` | `pylon_integer_get` / `pylon_integer_set` / `pylon_integer_range` | `Node::integer` / `set_integer` / `integer_range` |
| `IFloat::GetValue` / `SetValue` / `GetMin` / `GetMax` / `GetInc` | `pylon_float_get` / `pylon_float_set` / `pylon_float_range` | `Node::float` / `set_float` / `float_range` |
| `IBoolean::GetValue` / `SetValue` | `pylon_boolean_get` / `pylon_boolean_set` | `Node::boolean` / `set_boolean` |
| `ICommand::Execute` / `IsDone` | `pylon_command_execute` / `pylon_command_is_done` | `Node::execute` / `is_done` |
| `IEnumeration::GetEntries` | `pylon_enumeration_entries` | `Node::entries` |
| `IEnumEntry::GetSymbolic` | `pylon_enum_entry_symbolic` | `Node::symbolic` |
| `ICategory::GetFeatures` | `pylon_category_features` | `Node::features` |

### 像素类型、图像与格式转换

| pylon C++ API | C shim | Rust API |
|---|---|---|
| `EPixelType` | `PYLON_PIXEL_TYPE_*` | `PixelType::*` |
| `BitPerPixel`、`BitDepth`、`SamplesPerPixel`、`PlaneCount`、`IsMonoImage`（`IsColorImage` 与之互补）、`IsBayer`、`IsPacked`、`HasAlpha`、`CPixelTypeMapper::GetNameByPixelType` | `pylon_pixel_type_info` | `PixelType::info`、`PixelTypeInfo` |
| `CPixelTypeMapper::GetPylonPixelTypeByName` | `pylon_pixel_type_from_name` | `PixelType::from_name` |
| `ComputeStride` | `pylon_pixel_type_stride` | `PixelType::stride` |
| `CPylonImage()` 及其 `IImage` getter | `pylon_image_create` / `pylon_image_destroy` / `pylon_image_view` | `Image::new` / drop / `Image::view`、`ImageView`、`Orientation` |
| `CImageFormatConverter()` | `pylon_converter_create` / `pylon_converter_destroy` | `Converter::new` / drop |
| `CImageFormatConverter::GetNodeMap`（`Gamma`、`OutputBitAlignment`、`OutputOrientation` 等） | `pylon_converter_node_map` | `Converter::node_map` |
| `CImageFormatConverter::OutputPixelFormat` | `pylon_converter_set_output_pixel_type` | `Converter::set_output_pixel_type` |
| `CImageFormatConverter::ImageHasDestinationFormat` | `pylon_converter_has_destination_format` | `Converter::has_destination_format` |
| `CImageFormatConverter::Convert` | `pylon_converter_convert` | `Converter::convert`；源图像来自 `GrabResultInfo::image`、`Image::view` 或 `ImageView::new` |

## 暂不包装 / 无需包装

| pylon C++ API | 状态 | 说明 |
|---|---|---|
| GenDC 多组件数据：`CPylonDataContainer`、`CPylonDataComponent`、`GetDataComponent*` | 暂不包装 | GenDC payload 只通过 grab result 的图像暴露第一个图像组件 |
| Compression Beyond：`CImageDecompressor` | 暂不包装 | |
| 显式事件采集：`StartEventGrabbing`、`ProcessOneEvent`、`StopEventGrabbing`、`IsEventGrabbing`、`GetCameraEventWaitObject` | 暂不包装 | 采集期间的事件由 `GrabCameraEvents` 交付 |
| GigE 专属：Action Command、IP 配置、Multicast | 暂不包装 | |
| 其余 GenApi 成员：`INodeMap::GetNodes` / `Poll` / `InvalidateNodes` / `GetLock`、`IRegister`、`IPort`、单位、可见性、表示方式、列表步进 | 暂不包装 | 缺少 `GetLock` 时，selector 加 value 这类组合操作在多线程下不是原子的 |
| `CDeviceInfo` / `CGrabResultPtr` 的复制（Rust `Clone`） | 暂不包装 | shim 无复制函数；可重新枚举，或调用 `Camera::device_info` |
| converter 的 `InconvertibleEdgeHandling` 取 `Clip` / `Extend` | 拒绝 | pylon 在 `Clip` 下写出 planar 目标的范围，在 `Extend` 下读 16-bit Bayer 源起点之前的内存；参数不是 `SetZero` 时 `Converter::convert` 返回错误 |
| converter 输出 `PixelType_YUV422planar` / `PixelType_YUV420planar` | 拒绝 | pylon 对这两种输出有部分色度字节不写入；`Converter::convert` 对其返回错误 |
| `GenApi::Deregister` | 无需包装 | 已从 shim 删除：它不等待正在执行的回调，与其他线程上的回调并发时进程会中止，且其 handle 就是回调对象，重复调用是 use after free；注册随相机结束，在闭包中检查一个标志即可停止响应 |
| 其他 node map 上的节点回调（transport layer、stream / event grabber、InstantCamera、chunk 数据、converter） | 无需包装 | chunk node map 随 buffer 复用；frame grabber 的 transport layer 事件来自内部线程；相机事件的数据节点在相机的 node map 中 |
| PylonGUI（`CPylonImageWindow`、`DisplayImage`）、`CPylonBitmapImage` | 无需包装 | GUI 不在范围内 |
| `CImagePersistence`、`CPylonImage::Save` / `Load`、`CAviWriter`、`CVideoWriter` | 无需包装 | 图像文件由 Rust 侧处理 |
| Grab loop thread（`GrabLoop_ProvidedByInstantCamera`）、`CImageEventHandler` | 无需包装 | 由调用方自己的 Rust 线程调用 `Camera::retrieve_result` 即可实现 |
| 自定义的 `CConfigurationEventHandler` 子类、掉线回调 | 无需包装 | 调用失败后查询 `Camera::is_device_removed`，与 pylon DeviceRemovalHandling 示例一致 |
| `Attach` / `DetachDevice` / `DestroyDevice` / `HasOwnership` | 无需包装 | camera 在整个生命周期内持有 device，重连即重新创建 camera |
| Low Level API：`IPylonDevice`、`IStreamGrabber`、`IEventGrabber`、`ITransportLayer`、`CTlFactory::CreateTl` / `EnumerateTls`、`CInstantInterface` | 无需包装 | InstantCamera 已覆盖 |
| `CInstantCameraArray` | 无需包装 | 多相机由 Rust 侧组织 |
| `IBufferFactory`、`SetCameraContext` / `GetCameraContext`、`GetQueuedBufferCount`（已弃用） | 无需包装 | 属于高级用法或已弃用；队列计数是 InstantCamera node map 中的节点 |
| `GetSfncVersion`、`IsGigE` / `IsUsb` / `IsCameraLink` / `IsCxp`、`IsDeviceAccessible` | 无需包装 | 可由节点、设备信息的 key 或打开结果得到 |
| `C*Parameter` 辅助功能：value correction、`TrySet*`、`*OrDefault`、按百分比设值 | 无需包装 | 在 Rust 侧由 range 与 access mode 组合实现 |
| `CFeaturePersistence::Load` / `Save`（文件版） | 无需包装 | 字符串版 + Rust 文件读写 |
| `CPylonImage` 其余功能：`CopyImage`、`AttachUserBuffer`、`AttachGrabResultBuffer`、`GetAoi`、`GetPlane`、`ChangePixelType` | 无需包装 | Rust slice 可完成 buffer 视图与复制；需要按帧持有结果时，轮换使用多个 `Image` |
| `CImageFormatConverter` 其余功能：`Initialize`、`GetBufferSizeForConversion`、`IsSupportedInputFormat` / `IsSupportedOutputFormat`、写入原始 buffer 的 `Convert` 重载 | 无需包装 | 转换到复用的 `Image` 已覆盖；不支持的格式以错误返回 |
