# pylon-cpp-rs

[English](README.md) | 简体中文

Basler pylon C++ SDK（pylon 12）的 Rust 安全包装，底层是一层很薄的 C shim。

## 进度

| 层 | 路径 | 状态 |
|---|---|---|
| C shim | `pylon-sys/shim` | 已完成 |
| 原始绑定（`-sys`，bindgen） | `pylon-sys` | 已完成；已在 Windows 验证，Linux 与 macOS 尚未测试 |
| Rust 安全 API | crate 根目录 | 计划中 |

范围：相机操作与基础的像素、格式转换。GUI 相关功能不在范围内。

平台：Windows x64、Linux x86_64 / aarch64、macOS x86_64 / arm64。

## 目录

```text
src/lib.rs                      Rust 安全 API（crate pylon-cpp-rs）
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
- 相机的打开 / 关闭状态只由 `pylon_camera_open` / `pylon_camera_close` 改变，node map 不会在调用方不知情时失效。
- pylon 在其内部线程调用的 handler 一律不包装。采集在调用 `pylon_camera_retrieve_result` 的线程中进行，相机事件以节点回调的形式在该调用内触发。
- pylon 枚举和全部 `EPixelType` 值在头文件中镜像，并用 `static_assert` 校验。
- shim 的每个 enum 都以 `int32_t` 为固定底层类型（C23、C++11），ABI 在各平台一致；bindgen 将其生成为 `PylonStatus(pub i32)` 这样的 newtype，也能容纳列出常量之外的值。

## API 对照

Rust API 列将随安全包装一同加入。

### 运行时与错误

| pylon C++ API | C shim |
|---|---|
| `PylonInitialize` / `PylonTerminate` | `pylon_initialize` / `pylon_terminate` |
| `GetPylonVersion` | `pylon_version` |
| `GenericException` 及其子类 | `PylonStatus`、`pylon_last_error` |

### 设备

| pylon C++ API | C shim |
|---|---|
| `CTlFactory::EnumerateDevices`（带/不带过滤） | `pylon_enumerate_devices` |
| `CDeviceInfo()` | `pylon_device_info_create` / `pylon_device_info_destroy` |
| `CDeviceInfo::GetPropertyValue`、各 `Get<Key>` | `pylon_device_info_get` |
| `CDeviceInfo::SetPropertyValue`、各 `Set<Key>` | `pylon_device_info_set` |
| `CDeviceInfo::GetPropertyNames` | `pylon_device_info_keys` |

### 相机

| pylon C++ API | C shim |
|---|---|
| `CTlFactory::CreateDevice` / `CreateFirstDevice` + `CInstantCamera(IPylonDevice*)` | `pylon_camera_create` |
| `CInstantCamera::~CInstantCamera` | `pylon_camera_destroy` |
| `GetDeviceInfo` | `pylon_camera_device_info` |
| `Open` / `Close` / `IsOpen` | `pylon_camera_open` / `pylon_camera_close` / `pylon_camera_is_open` |
| `IsCameraDeviceRemoved` | `pylon_camera_is_device_removed` |
| `GetNodeMap` / `GetTLNodeMap` / `GetStreamGrabberNodeMap` / `GetEventGrabberNodeMap` / `GetInstantCameraNodeMap` | `pylon_camera_node_map` |
| `RegisterConfiguration` 配合 `CAcquireContinuousConfiguration` / `CAcquireSingleFrameConfiguration` / `CSoftwareTriggerConfiguration` | `pylon_camera_set_configuration` |
| `StartGrabbing`（两个重载） | `pylon_camera_start_grabbing` |
| `StopGrabbing` / `IsGrabbing` | `pylon_camera_stop_grabbing` / `pylon_camera_is_grabbing` |
| `RetrieveResult` | `pylon_camera_retrieve_result` |
| `GrabOne` | `pylon_camera_grab_one` |
| `CanWaitForFrameTriggerReady` / `WaitForFrameTriggerReady` | `pylon_camera_can_wait_for_frame_trigger_ready` / `pylon_camera_wait_for_frame_trigger_ready` |
| `ExecuteSoftwareTrigger` | `pylon_camera_execute_software_trigger` |
| InstantCamera 参数（`MaxNumBuffer`、`OutputQueueSize`、`GrabCameraEvents` 等） | `PYLON_NODE_MAP_INSTANT_CAMERA` 上的节点 API |

### Grab result

| pylon C++ API | C shim |
|---|---|
| `CGrabResultPtr` 复制 / 释放 | `PylonGrabResult` / `pylon_grab_result_destroy` |
| `CGrabResultData` 各 getter 与 `operator IImage&` | `pylon_grab_result_info` |
| `GetErrorDescription` | `pylon_grab_result_error_description` |
| `GetChunkDataNodeMap` | `pylon_grab_result_chunk_node_map` |

### Node map 与节点

| pylon C++ API | C shim |
|---|---|
| `INodeMap::GetNode` | `pylon_node_map_node` |
| `CFeaturePersistence::LoadFromString` / `SaveToString` | `pylon_node_map_load` / `pylon_node_map_save` |
| `INode::GetPrincipalInterfaceType` | `pylon_node_type` |
| `IBase::GetAccessMode`（`IsReadable` / `IsWritable` / `IsAvailable`） | `pylon_node_access_mode` |
| `INode::GetName` / `GetDisplayName` / `GetToolTip` / `GetDescription` | `pylon_node_text` |
| `GenApi::Register` / `GenApi::Deregister`、`CCameraEventHandler` | `pylon_node_register_callback` / `pylon_node_deregister_callback` |
| `IValue::ToString` / `FromString`（含枚举与字符串的取值、设值） | `pylon_value_to_string` / `pylon_value_from_string` |
| `IInteger::GetValue` / `SetValue` / `GetMin` / `GetMax` / `GetInc` | `pylon_integer_get` / `pylon_integer_set` / `pylon_integer_range` |
| `IFloat::GetValue` / `SetValue` / `GetMin` / `GetMax` / `GetInc` | `pylon_float_get` / `pylon_float_set` / `pylon_float_range` |
| `IBoolean::GetValue` / `SetValue` | `pylon_boolean_get` / `pylon_boolean_set` |
| `ICommand::Execute` / `IsDone` | `pylon_command_execute` / `pylon_command_is_done` |
| `IEnumeration::GetEntries` | `pylon_enumeration_entries` |
| `IEnumEntry::GetSymbolic` | `pylon_enum_entry_symbolic` |
| `ICategory::GetFeatures` | `pylon_category_features` |

### 像素类型、图像与格式转换

| pylon C++ API | C shim |
|---|---|
| `EPixelType` | `PYLON_PIXEL_TYPE_*` |
| `BitPerPixel`、`BitDepth`、`SamplesPerPixel`、`PlaneCount`、`IsMonoImage`（`IsColorImage` 与之互补）、`IsBayer`、`IsPacked`、`HasAlpha`、`CPixelTypeMapper::GetNameByPixelType` | `pylon_pixel_type_info` |
| `CPixelTypeMapper::GetPylonPixelTypeByName` | `pylon_pixel_type_from_name` |
| `ComputeStride` | `pylon_pixel_type_stride` |
| `CPylonImage()` 及其 `IImage` getter | `pylon_image_create` / `pylon_image_destroy` / `pylon_image_view` |
| `CImageFormatConverter()` | `pylon_converter_create` / `pylon_converter_destroy` |
| `CImageFormatConverter::GetNodeMap`（`Gamma`、`OutputBitAlignment`、`OutputOrientation` 等） | `pylon_converter_node_map` |
| `CImageFormatConverter::OutputPixelFormat` | `pylon_converter_set_output_pixel_type` |
| `CImageFormatConverter::ImageHasDestinationFormat` | `pylon_converter_has_destination_format` |
| `CImageFormatConverter::Convert` | `pylon_converter_convert` |

## 暂不包装 / 无需包装

| pylon C++ API | 状态 | 说明 |
|---|---|---|
| GenDC 多组件数据：`CPylonDataContainer`、`CPylonDataComponent`、`GetDataComponent*` | 暂不包装 | GenDC payload 只通过 grab result 的图像暴露第一个图像组件 |
| Compression Beyond：`CImageDecompressor` | 暂不包装 | |
| 显式事件采集：`StartEventGrabbing`、`ProcessOneEvent`、`StopEventGrabbing`、`IsEventGrabbing`、`GetCameraEventWaitObject` | 暂不包装 | 采集期间的事件由 `GrabCameraEvents` 交付 |
| GigE 专属：Action Command、IP 配置、Multicast | 暂不包装 | |
| 其余 GenApi 成员：`INodeMap::GetNodes` / `Poll` / `InvalidateNodes`、`IRegister`、`IPort`、单位、可见性、表示方式、列表步进 | 暂不包装 | |
| PylonGUI（`CPylonImageWindow`、`DisplayImage`）、`CPylonBitmapImage` | 无需包装 | GUI 不在范围内 |
| `CImagePersistence`、`CPylonImage::Save` / `Load`、`CAviWriter`、`CVideoWriter` | 无需包装 | 图像文件由 Rust 侧处理 |
| Grab loop thread（`GrabLoop_ProvidedByInstantCamera`）、`CImageEventHandler` | 无需包装 | 由调用方自己的 Rust 线程调用 `pylon_camera_retrieve_result` 即可实现 |
| 自定义的 `CConfigurationEventHandler` 子类、掉线回调 | 无需包装 | 调用失败后查询 `pylon_camera_is_device_removed`，与 pylon DeviceRemovalHandling 示例一致 |
| `Attach` / `DetachDevice` / `DestroyDevice` / `HasOwnership` | 无需包装 | camera 在整个生命周期内持有 device，重连即重新创建 camera |
| Low Level API：`IPylonDevice`、`IStreamGrabber`、`IEventGrabber`、`ITransportLayer`、`CTlFactory::CreateTl` / `EnumerateTls`、`CInstantInterface` | 无需包装 | InstantCamera 已覆盖 |
| `CInstantCameraArray` | 无需包装 | 多相机由 Rust 侧组织 |
| `IBufferFactory`、`SetCameraContext` / `GetCameraContext`、`GetQueuedBufferCount`（已弃用） | 无需包装 | 属于高级用法或已弃用；队列计数是 InstantCamera node map 中的节点 |
| `GetSfncVersion`、`IsGigE` / `IsUsb` / `IsCameraLink` / `IsCxp`、`IsDeviceAccessible` | 无需包装 | 可由节点、设备信息的 key 或打开结果得到 |
| `C*Parameter` 辅助功能：value correction、`TrySet*`、`*OrDefault`、按百分比设值 | 无需包装 | 在 Rust 侧由 range 与 access mode 组合实现 |
| `CFeaturePersistence::Load` / `Save`（文件版） | 无需包装 | 字符串版 + Rust 文件读写 |
| `CPylonImage` 其余功能：`CopyImage`、`AttachUserBuffer`、`AttachGrabResultBuffer`、`GetAoi`、`GetPlane`、`ChangePixelType` | 无需包装 | Rust slice 可完成 buffer 视图与复制；需要按帧持有结果时，轮换使用多个 `PylonImage` |
| `CImageFormatConverter` 其余功能：`Initialize`、`GetBufferSizeForConversion`、`IsSupportedInputFormat` / `IsSupportedOutputFormat`、写入原始 buffer 的 `Convert` 重载 | 无需包装 | 转换到复用的 `PylonImage` 已覆盖；不支持的格式以错误返回 |
