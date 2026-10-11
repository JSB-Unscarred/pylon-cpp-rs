//! Devices, cameras and grab results.

use crate::{
    Error, ErrorKind, ImageView, Node, NodeMap, Pylon, Result, bytes, c_string, check, handle,
    optional_handle, out, push_handle, string, strings, sys,
};
use std::ffi::c_void;
use std::ptr::{self, NonNull};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// `Pylon::CDeviceInfo`, a property table keyed by the names of `Pylon::Key`, e.g. SerialNumber,
/// ModelName, UserDefinedName, FullName, DeviceClass. Used as enumeration result, as enumeration
/// filter and to select the device of a [`Camera`].
pub struct DeviceInfo {
    raw: NonNull<sys::PylonDeviceInfo>,
    _pylon: Pylon,
}

// SAFETY: CDeviceInfo copies own their properties and have no thread affinity; DeviceInfo is not
// Sync because it is unsynchronized (pylon_shim.h Threads).
unsafe impl Send for DeviceInfo {}

impl DeviceInfo {
    /// Empty device info.
    pub fn new(pylon: &Pylon) -> Result<DeviceInfo> {
        // SAFETY: the out parameter is a valid handle pointer; pylon_device_info_create writes a
        // non-NULL handle on PYLON_OK (pylon_shim.h).
        let raw = unsafe { handle(|info| sys::pylon_device_info_create(info)) }?;
        Ok(DeviceInfo { raw, _pylon: pylon.clone() })
    }

    /// Value of a property; None if the device info has no such property.
    pub fn get(&self, key: &str) -> Result<Option<String>> {
        let key = c_string(key)?;
        // SAFETY: raw is a live device info; key is a C string; cb and ctx come from `strings`.
        let texts = strings(|cb, ctx| unsafe {
            sys::pylon_device_info_get(self.raw.as_ptr(), key.as_ptr(), cb, ctx)
        });
        Ok(texts?.pop())
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        let (key, value) = (c_string(key)?, c_string(value)?);
        // SAFETY: raw is a live device info; key and value are C strings.
        check(unsafe {
            sys::pylon_device_info_set(self.raw.as_ptr(), key.as_ptr(), value.as_ptr())
        })
    }

    /// Names of the properties that are set.
    pub fn keys(&self) -> Result<Vec<String>> {
        // SAFETY: raw is a live device info; cb and ctx come from `strings`.
        strings(|cb, ctx| unsafe { sys::pylon_device_info_keys(self.raw.as_ptr(), cb, ctx) })
    }
}

impl Drop for DeviceInfo {
    fn drop(&mut self) {
        // SAFETY: raw is a live device info, destroyed once.
        unsafe { sys::pylon_device_info_destroy(self.raw.as_ptr()) };
    }
}

impl Pylon {
    /// `CTlFactory::EnumerateDevices`: the devices matching any of the filters, or all devices if
    /// there are no filters.
    pub fn enumerate(&self, filters: &[&DeviceInfo]) -> Result<Vec<DeviceInfo>> {
        let filters: Vec<_> = filters.iter().map(|info| info.raw.as_ptr().cast_const()).collect();
        let mut found = Vec::<NonNull<sys::PylonDeviceInfo>>::new();
        // SAFETY: filters holds filter_count live device infos; found is the vector push_handle
        // expects.
        let status = unsafe {
            sys::pylon_enumerate_devices(
                filters.as_ptr(),
                filters.len(),
                Some(push_handle),
                (&raw mut found).cast(),
            )
        };
        // Wrapping owns the passed device infos, also those passed before a failure.
        let found = found.into_iter().map(|raw| DeviceInfo { raw, _pylon: self.clone() }).collect();
        check(status).map(|()| found)
    }
}

/// Configuration applied when the camera opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum Configuration {
    /// No configuration: the camera keeps its settings.
    None = sys::PylonConfiguration::PYLON_CONFIGURATION_NONE.0,
    /// `CAcquireContinuousConfiguration`, the pylon default.
    AcquireContinuous = sys::PylonConfiguration::PYLON_CONFIGURATION_ACQUIRE_CONTINUOUS.0,
    /// `CAcquireSingleFrameConfiguration`.
    AcquireSingleFrame = sys::PylonConfiguration::PYLON_CONFIGURATION_ACQUIRE_SINGLE_FRAME.0,
    /// `CSoftwareTriggerConfiguration`.
    SoftwareTrigger = sys::PylonConfiguration::PYLON_CONFIGURATION_SOFTWARE_TRIGGER.0,
}

/// `Pylon::EGrabStrategy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum GrabStrategy {
    OneByOne = sys::PylonGrabStrategy::PYLON_GRAB_STRATEGY_ONE_BY_ONE.0,
    LatestImageOnly = sys::PylonGrabStrategy::PYLON_GRAB_STRATEGY_LATEST_IMAGE_ONLY.0,
    LatestImages = sys::PylonGrabStrategy::PYLON_GRAB_STRATEGY_LATEST_IMAGES.0,
    UpcomingImage = sys::PylonGrabStrategy::PYLON_GRAB_STRATEGY_UPCOMING_IMAGE.0,
}

/// `Pylon::CInstantCamera` with its device. The camera is closed when it is dropped; to change
/// settings that apply only when opening (configuration, GrabCameraEvents), drop the camera and
/// create it again.
pub struct Camera {
    raw: NonNull<sys::PylonCamera>,
    /// [`thread_id`] of the thread waiting in retrieve_result or grab_one, 0 if none.
    waiter: AtomicUsize,
    /// stop_grabbing was called by a callback of the waiter; handled when its call returns.
    stop_requested: AtomicBool,
    pylon: Pylon,
}

// SAFETY: pylon synchronizes PylonCamera and it has no thread affinity (pylon_shim.h Threads); the
// one-waiter rule is enforced by `waiter`, and destruction needs ownership.
unsafe impl Send for Camera {}
// SAFETY: as for Send.
unsafe impl Sync for Camera {}

impl Camera {
    /// Creates the camera of the first device matching `device`, or of the first device found if
    /// None. The camera is closed.
    pub fn new(pylon: &Pylon, device: Option<&DeviceInfo>) -> Result<Camera> {
        let info = device.map_or(ptr::null(), |info| info.raw.as_ptr().cast_const());
        // SAFETY: info is NULL or a live device info; the out parameter is a valid handle pointer;
        // pylon_camera_create writes a non-NULL handle on PYLON_OK (pylon_shim.h).
        let raw = unsafe { handle(|camera| sys::pylon_camera_create(info, camera)) }?;
        Ok(Camera {
            raw,
            waiter: AtomicUsize::new(0),
            stop_requested: AtomicBool::new(false),
            pylon: pylon.clone(),
        })
    }

    /// Copy of the device info of the camera.
    pub fn device_info(&self) -> Result<DeviceInfo> {
        // SAFETY: raw is a live camera; the out parameter is a valid handle pointer;
        // pylon_camera_device_info writes a non-NULL handle on PYLON_OK (pylon_shim.h).
        let raw = unsafe { handle(|info| sys::pylon_camera_device_info(self.raw.as_ptr(), info)) }?;
        Ok(DeviceInfo { raw, _pylon: self.pylon.clone() })
    }

    /// `CInstantCamera::Open`; applies the configuration. Opening an open camera does nothing.
    pub fn open(&self) -> Result<()> {
        // SAFETY: raw is a live camera.
        check(unsafe { sys::pylon_camera_open(self.raw.as_ptr()) })
    }

    pub fn is_open(&self) -> bool {
        // SAFETY: raw is a live camera.
        unsafe { sys::pylon_camera_is_open(self.raw.as_ptr()) }
    }

    /// `CInstantCamera::IsCameraDeviceRemoved`; detection requires an open camera. After a removal,
    /// drop the camera and create it again.
    pub fn is_device_removed(&self) -> bool {
        // SAFETY: raw is a live camera.
        unsafe { sys::pylon_camera_is_device_removed(self.raw.as_ptr()) }
    }

    /// Camera features (`GetNodeMap`); their values can be accessed while the camera is open.
    pub fn node_map(&self) -> Result<NodeMap<'_>> {
        self.map(sys::PylonNodeMapKind::PYLON_NODE_MAP_DEVICE)
    }

    /// `GetTLNodeMap`.
    pub fn tl_node_map(&self) -> Result<NodeMap<'_>> {
        self.map(sys::PylonNodeMapKind::PYLON_NODE_MAP_TRANSPORT_LAYER)
    }

    /// `GetStreamGrabberNodeMap`; requires an open camera.
    pub fn stream_grabber_node_map(&self) -> Result<NodeMap<'_>> {
        self.map(sys::PylonNodeMapKind::PYLON_NODE_MAP_STREAM_GRABBER)
    }

    /// `GetEventGrabberNodeMap`; requires an open camera.
    pub fn event_grabber_node_map(&self) -> Result<NodeMap<'_>> {
        self.map(sys::PylonNodeMapKind::PYLON_NODE_MAP_EVENT_GRABBER)
    }

    /// `GetInstantCameraNodeMap`: MaxNumBuffer, OutputQueueSize, GrabCameraEvents, ...
    pub fn instant_camera_node_map(&self) -> Result<NodeMap<'_>> {
        self.map(sys::PylonNodeMapKind::PYLON_NODE_MAP_INSTANT_CAMERA)
    }

    fn map(&self, kind: sys::PylonNodeMapKind) -> Result<NodeMap<'_>> {
        // SAFETY: raw is a live camera; the out parameter is a valid handle pointer;
        // pylon_camera_node_map writes a non-NULL handle on PYLON_OK (pylon_shim.h).
        let map =
            unsafe { handle(|map| sys::pylon_camera_node_map(self.raw.as_ptr(), kind, map)) }?;
        // SAFETY: camera node maps stay valid until pylon_camera_close or pylon_camera_destroy, and
        // a failed open does not end them (pylon_shim.h). This crate never calls
        // pylon_camera_close, and destroys the camera only in Drop, which needs ownership, so the
        // map outlives the borrow of self.
        Ok(unsafe { NodeMap::from_raw(map) })
    }

    /// `CInstantCamera::RegisterConfiguration` with `RegistrationMode_ReplaceAll`. Takes effect when
    /// the camera opens, so call it before [`open`](Self::open),
    /// [`start_grabbing`](Self::start_grabbing) or [`grab_one`](Self::grab_one); on an open camera
    /// it returns Ok but has no effect.
    pub fn set_configuration(&self, configuration: Configuration) -> Result<()> {
        let configuration = sys::PylonConfiguration(configuration as i32);
        // SAFETY: raw is a live camera.
        check(unsafe { sys::pylon_camera_set_configuration(self.raw.as_ptr(), configuration) })
    }

    /// Opens the camera, then `CInstantCamera::StartGrabbing`; the camera stays open when grabbing
    /// stops. `max_images` 0 grabs until [`stop_grabbing`](Self::stop_grabbing), otherwise grabbing
    /// stops after `max_images` results have been retrieved.
    pub fn start_grabbing(&self, strategy: GrabStrategy, max_images: usize) -> Result<()> {
        let strategy = sys::PylonGrabStrategy(strategy as i32);
        // SAFETY: raw is a live camera.
        check(unsafe { sys::pylon_camera_start_grabbing(self.raw.as_ptr(), strategy, max_images) })
    }

    /// `CInstantCamera::StopGrabbing`. A thread waiting in
    /// [`retrieve_result`](Self::retrieve_result) returns once a callback running there has
    /// returned, with the result it already has or with None. Called from a callback running
    /// inside `retrieve_result`, it stops grabbing when that call returns, i.e. with its result or
    /// timeout; inside [`grab_one`](Self::grab_one) it does nothing, since `grab_one` stops by
    /// itself.
    pub fn stop_grabbing(&self) {
        if self.waiter.load(Ordering::Relaxed) == thread_id() {
            // pylon crashes when grabbing stops inside its own RetrieveResult.
            self.stop_requested.store(true, Ordering::Relaxed);
            return;
        }
        // SAFETY: raw is a live camera.
        unsafe { sys::pylon_camera_stop_grabbing(self.raw.as_ptr()) }
    }

    pub fn is_grabbing(&self) -> bool {
        // SAFETY: raw is a live camera.
        unsafe { sys::pylon_camera_is_grabbing(self.raw.as_ptr()) }
    }

    /// `CInstantCamera::RetrieveResult`. None if the timeout expired or the camera is not grabbing.
    /// Callbacks of camera events run inside this call. Fails with [`ErrorKind::Runtime`] while
    /// another call waits in this method or in [`grab_one`](Self::grab_one), also when called from
    /// a callback running inside such a call.
    pub fn retrieve_result(&self, timeout_ms: u32) -> Result<Option<GrabResult>> {
        // SAFETY: raw is a live camera; the out parameter is a valid handle pointer; `wait` admits
        // one thread at a time.
        self.wait(false, |result| unsafe {
            sys::pylon_camera_retrieve_result(self.raw.as_ptr(), timeout_ms, result)
        })
    }

    /// Opens the camera, then `CInstantCamera::GrabOne`: grabs one image, also stopping on timeout.
    /// None if the timeout expired. Fails with [`ErrorKind::Runtime`] while another call waits in
    /// this method or in [`retrieve_result`](Self::retrieve_result), also when called from a
    /// callback running inside such a call.
    pub fn grab_one(&self, timeout_ms: u32) -> Result<Option<GrabResult>> {
        // SAFETY: raw is a live camera; the out parameter is a valid handle pointer; `wait` admits
        // one thread at a time.
        self.wait(true, |result| unsafe {
            sys::pylon_camera_grab_one(self.raw.as_ptr(), timeout_ms, result)
        })
    }

    /// Runs a call that waits for a result; only one thread at a time may wait (pylon_shim.h).
    /// Afterwards it stops grabbing if a callback of the call asked for it, unless the call
    /// `stops_itself`.
    fn wait(
        &self,
        stops_itself: bool,
        call: impl FnOnce(&mut *mut sys::PylonGrabResult) -> sys::PylonStatus,
    ) -> Result<Option<GrabResult>> {
        let thread = thread_id();
        if self.waiter.compare_exchange(0, thread, Ordering::Acquire, Ordering::Relaxed).is_err() {
            return Err(Error::new(ErrorKind::Runtime, "a call is already waiting for a result"));
        }
        let result = optional_handle(call);
        let stop = self.stop_requested.swap(false, Ordering::Relaxed);
        self.waiter.store(0, Ordering::Release);
        if stop && !stops_itself {
            self.stop_grabbing();
        }
        result.map(|raw| raw.map(|raw| GrabResult { raw, _pylon: self.pylon.clone() }))
    }

    /// `CInstantCamera::CanWaitForFrameTriggerReady`.
    pub fn can_wait_for_frame_trigger_ready(&self) -> Result<bool> {
        // SAFETY: raw is a live camera; the out parameter is a valid bool.
        out(|ready| unsafe {
            sys::pylon_camera_can_wait_for_frame_trigger_ready(self.raw.as_ptr(), ready)
        })
    }

    /// `CInstantCamera::WaitForFrameTriggerReady`; false if the timeout expired.
    pub fn wait_for_frame_trigger_ready(&self, timeout_ms: u32) -> Result<bool> {
        // SAFETY: raw is a live camera; the out parameter is a valid bool.
        out(|ready| unsafe {
            sys::pylon_camera_wait_for_frame_trigger_ready(self.raw.as_ptr(), timeout_ms, ready)
        })
    }

    /// `CInstantCamera::ExecuteSoftwareTrigger`.
    pub fn execute_software_trigger(&self) -> Result<()> {
        // SAFETY: raw is a live camera.
        check(unsafe { sys::pylon_camera_execute_software_trigger(self.raw.as_ptr()) })
    }

    /// Runs `f` with the node map and the node whenever the node of the camera node map named
    /// `name` may have changed, also when the same value is written again or a node it depends on
    /// changes. Fails with [`ErrorKind::InvalidArgument`] if there is no such node.
    ///
    /// `f` runs on the thread that changes the node, possibly on several threads at once. Camera
    /// events (GrabCameraEvents enabled in the instant camera node map before opening) update their
    /// data nodes inside [`retrieve_result`](Self::retrieve_result) and
    /// [`grab_one`](Self::grab_one), so their callbacks run there. Register after opening, before
    /// sharing the camera: registering while callbacks run is not verified to be safe.
    ///
    /// Registrations last until the camera is dropped; `f` may still run during that drop, with
    /// event data nodes unreadable, and is dropped inside it. To stop reacting, check a flag in
    /// `f`. A panic in `f` aborts the process.
    ///
    /// When `f` runs inside `retrieve_result` or `grab_one`, these two methods of the same camera
    /// fail inside `f`, and [`stop_grabbing`](Self::stop_grabbing) takes effect when that call
    /// returns; when `f` runs inside [`start_grabbing`](Self::start_grabbing), `stop_grabbing`
    /// deadlocks. Capture a `Weak<Camera>` if `f` needs the camera, since an `Arc` keeps it alive
    /// forever. Read the data you need and send it to your own thread, ignoring send errors, since
    /// the receiver may be gone while the camera drops.
    pub fn register_callback<F>(&mut self, name: &str, f: F) -> Result<()>
    where
        F: Fn(NodeMap<'_>, Node<'_>) + Send + Sync + 'static,
    {
        let map = self.node_map()?;
        let Some(node) = map.node(name)? else {
            return Err(Error::new(ErrorKind::InvalidArgument, format!("node {name} not found")));
        };
        let ctx = Box::into_raw(Box::new(Callback { map: map.raw, f }));
        // SAFETY: node belongs to the camera node map and is valid during the call; &mut self keeps
        // other threads from changing nodes of the camera meanwhile. ctx passes to the shim in
        // every case; the shim calls drop_callback::<F> at most once, after the last on_node::<F>
        // (pylon_shim.h).
        check(unsafe {
            sys::pylon_node_register_callback(
                node.raw.as_ptr(),
                Some(on_node::<F>),
                ctx.cast(),
                Some(drop_callback::<F>),
            )
        })
    }
}

impl Drop for Camera {
    fn drop(&mut self) {
        // SAFETY: raw is a live camera, destroyed once; ownership excludes other users.
        unsafe { sys::pylon_camera_destroy(self.raw.as_ptr()) };
    }
}

/// Nonzero number that identifies the calling thread among the running threads.
fn thread_id() -> usize {
    thread_local!(static ID: u8 = const { 0 });
    ID.with(|id| ptr::from_ref(id).addr())
}

/// Context of a registration; GenApi owns it from registration until the node map ends.
struct Callback<F> {
    map: NonNull<sys::PylonNodeMap>,
    f: F,
}

/// # Safety
/// `ctx` is a live `Callback<F>` of [`Camera::register_callback`]; `node` belongs to its map.
unsafe extern "C" fn on_node<F: Fn(NodeMap<'_>, Node<'_>)>(
    ctx: *mut c_void,
    node: *mut sys::PylonNode,
) {
    // SAFETY: the shim calls this only before drop_callback; registrations may run on several
    // threads at once, so only a shared reference is formed (register_callback requires F: Sync).
    let callback = unsafe { &*ctx.cast::<Callback<F>>() };
    // SAFETY: the registration ends with the node map, so the map and node are valid during the
    // call; node is non-NULL (pylon_shim.h). The views cannot leave the call: F accepts any
    // lifetime.
    let (map, node) =
        unsafe { (NodeMap::from_raw(callback.map), Node::from_raw(NonNull::new_unchecked(node))) };
    (callback.f)(map, node);
}

/// # Safety
/// `ctx` is a `Callback<F>` of [`Camera::register_callback`]; it is released once.
unsafe extern "C" fn drop_callback<F>(ctx: *mut c_void) {
    // SAFETY: the shim calls this once, after the last on_node; register_callback requires F: Send,
    // so it may be dropped on any thread.
    drop(unsafe { Box::from_raw(ctx.cast::<Callback<F>>()) });
}

/// One grab result, `Pylon::CGrabResultPtr`. It may outlive its camera; while it lives, its buffer
/// stays out of the grab queue.
pub struct GrabResult {
    raw: NonNull<sys::PylonGrabResult>,
    _pylon: Pylon,
}

// SAFETY: grab results may be used from any thread (pylon_shim.h Threads); GrabResult is not Sync
// because concurrent calls on one CGrabResultPtr are not documented.
unsafe impl Send for GrabResult {}

impl GrabResult {
    /// Properties and buffers of the result.
    pub fn info(&self) -> Result<GrabResultInfo<'_>> {
        // SAFETY: raw is a live result; the out parameter is a valid PylonGrabResultInfo.
        let info = out(|info| unsafe { sys::pylon_grab_result_info(self.raw.as_ptr(), info) })?;
        // SAFETY: the buffers stay valid until the result is destroyed (pylon_shim.h), which needs
        // ownership, so for the borrow of self. Only chunk nodes write into them, and the chunk
        // node map needs `&mut self`.
        let (payload, image) =
            unsafe { (bytes(info.payload, info.payload_size), ImageView::from_raw(&info.image)) };
        Ok(GrabResultInfo {
            succeeded: info.succeeded,
            error_code: info.error_code,
            payload_type: PayloadType(info.payload_type),
            payload,
            image,
            offset_x: info.offset_x,
            offset_y: info.offset_y,
            padding_y: info.padding_y,
            block_id: info.block_id,
            timestamp: info.timestamp,
            id: info.id,
            image_number: info.image_number,
            skipped_images: info.skipped_images,
        })
    }

    /// `CGrabResultData::GetErrorDescription`.
    pub fn error_description(&self) -> Result<String> {
        // SAFETY: raw is a live result; cb and ctx come from `string`.
        string(|cb, ctx| unsafe {
            sys::pylon_grab_result_error_description(self.raw.as_ptr(), cb, ctx)
        })
    }

    /// `CGrabResultData::GetChunkDataNodeMap`; empty if chunks are disabled. Writable chunk nodes
    /// write into the buffer of the result, so the map borrows the result mutably; read the chunk
    /// values before calling [`info`](Self::info).
    pub fn chunk_node_map(&mut self) -> Result<NodeMap<'_>> {
        // SAFETY: raw is a live result; the out parameter is a valid handle pointer;
        // pylon_grab_result_chunk_node_map writes a non-NULL handle on PYLON_OK (pylon_shim.h).
        let map =
            unsafe { handle(|map| sys::pylon_grab_result_chunk_node_map(self.raw.as_ptr(), map)) }?;
        // SAFETY: the map stays valid until the result is destroyed (pylon_shim.h), which needs
        // ownership, so for the borrow of self.
        Ok(unsafe { NodeMap::from_raw(map) })
    }
}

impl Drop for GrabResult {
    fn drop(&mut self) {
        // SAFETY: raw is a live result, destroyed once.
        unsafe { sys::pylon_grab_result_destroy(self.raw.as_ptr()) };
    }
}

/// Properties and buffers of a [`GrabResult`], `CGrabResultData`.
#[derive(Clone, Copy)]
pub struct GrabResultInfo<'a> {
    pub succeeded: bool,
    pub error_code: u32,
    pub payload_type: PayloadType,
    /// Whole payload including chunk data.
    pub payload: &'a [u8],
    /// The result as image; None if the grab failed. The first image component of a GenDC payload.
    pub image: Option<ImageView<'a>>,
    pub offset_x: u32,
    pub offset_y: u32,
    pub padding_y: u32,
    pub block_id: u64,
    pub timestamp: u64,
    pub id: i64,
    pub image_number: i64,
    pub skipped_images: i64,
}

/// `Pylon::EPayloadType`; pylon may report values outside the constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PayloadType(sys::PylonPayloadType);

impl PayloadType {
    pub const UNDEFINED: Self = Self(sys::PylonPayloadType::PYLON_PAYLOAD_TYPE_UNDEFINED);
    pub const IMAGE: Self = Self(sys::PylonPayloadType::PYLON_PAYLOAD_TYPE_IMAGE);
    pub const RAW_DATA: Self = Self(sys::PylonPayloadType::PYLON_PAYLOAD_TYPE_RAW_DATA);
    pub const FILE: Self = Self(sys::PylonPayloadType::PYLON_PAYLOAD_TYPE_FILE);
    pub const CHUNK_DATA: Self = Self(sys::PylonPayloadType::PYLON_PAYLOAD_TYPE_CHUNK_DATA);
    pub const GEN_DC: Self = Self(sys::PylonPayloadType::PYLON_PAYLOAD_TYPE_GEN_DC);
    pub const DEVICE_SPECIFIC: Self =
        Self(sys::PylonPayloadType::PYLON_PAYLOAD_TYPE_DEVICE_SPECIFIC);
}
