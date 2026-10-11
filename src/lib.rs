//! Safe Rust API for the Basler pylon C++ SDK.
//!
//! # Runtime
//!
//! [`Pylon::new`] initializes the pylon runtime of the process. At most one runtime exists at a
//! time: while one exists, `Pylon::new` fails and further references are clones of [`Pylon`].
//! Every owned object ([`DeviceInfo`], [`Camera`], [`GrabResult`], [`Image`], [`Converter`]) holds
//! a reference, and dropping the last reference terminates pylon, so this crate never initializes
//! or terminates pylon concurrently. Create one `Pylon` per program and pass `&Pylon` to library
//! code. Code that calls pylon directly is not covered by this rule. Pylon objects must not live in
//! `thread_local!` storage: pylon must not be terminated while Windows holds the loader lock.
//! Leaking a pylon object, e.g. with `mem::forget` or in a `static`, keeps pylon initialized until
//! the process exits; `Pylon::new` then keeps failing, which is safe.
//!
//! [`PixelType`] and [`version`] work without a runtime (verified on Windows).
//!
//! # Threads and borrowing
//!
//! [`Camera`] is `Send + Sync`: pylon synchronizes it, so a worker thread can grab while another
//! one stops grabbing; dropping it closes it. [`DeviceInfo`], [`GrabResult`], [`Image`] and
//! [`Converter`] are `Send`. Node maps, nodes, image views and grab result buffers borrow the
//! object they come from, so the borrow checker keeps them valid, and stay on its thread. GenApi
//! serializes single node operations, but sequences such as selector plus value are not atomic.
//! The rules for waiting, stopping and callbacks are on [`Camera::retrieve_result`],
//! [`Camera::stop_grabbing`] and [`Camera::register_callback`].
//!
//! # Strings, timeouts and errors
//!
//! Strings from pylon are decoded lossily as UTF-8. Timeouts are milliseconds; [`INFINITE`] waits
//! forever. [`Error`] carries the description pylon gives, or this crate's own for a failure it
//! detects itself.

#![deny(clippy::undocumented_unsafe_blocks)]

mod camera;
mod image;
mod node;

pub use camera::{
    Camera, Configuration, DeviceInfo, GrabResult, GrabResultInfo, GrabStrategy, PayloadType,
};
pub use image::{Converter, Image, ImageView, Orientation, PixelType, PixelTypeInfo};
pub use node::{AccessMode, Node, NodeMap, NodeType};

use pylon_sys as sys;
use std::ffi::{CStr, CString, c_char, c_void};
use std::fmt;
use std::ptr::{self, NonNull};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Timeout that waits forever.
pub const INFINITE: u32 = sys::PYLON_INFINITE;

/// Version of the pylon runtime as (major, minor, subminor, build).
pub fn version() -> (u32, u32, u32, u32) {
    // SAFETY: GetPylonVersion has no preconditions.
    let version = unsafe { sys::pylon_version() };
    (version.major, version.minor, version.subminor, version.build)
}

/// Reference to the pylon runtime; see the [crate documentation](crate#runtime).
#[derive(Clone)]
pub struct Pylon {
    _runtime: Arc<Runtime>,
}

/// The one pylon initialization of this crate; terminates pylon when dropped.
struct Runtime;

/// True while a [`Runtime`] exists, including while it terminates pylon.
static ALIVE: AtomicBool = AtomicBool::new(false);

impl Pylon {
    /// Initializes the pylon runtime. Fails with [`ErrorKind::Runtime`] while a runtime exists,
    /// including while the last reference to it terminates pylon on another thread.
    pub fn new() -> Result<Pylon> {
        if ALIVE.swap(true, Ordering::Acquire) {
            return Err(Error::new(ErrorKind::Runtime, "a pylon runtime already exists"));
        }
        // SAFETY: ALIVE was false, so this crate holds no initialization and no other thread
        // initializes or terminates pylon through it.
        let status = unsafe { sys::pylon_initialize() };
        check(status).inspect_err(|_| ALIVE.store(false, Ordering::Release))?;
        Ok(Pylon { _runtime: Arc::new(Runtime) })
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        // SAFETY: pairs with the initialization in Pylon::new. Every object of this crate holds a
        // Pylon and is destroyed before its fields drop, so no object is left.
        unsafe { sys::pylon_terminate() };
        ALIVE.store(false, Ordering::Release);
    }
}

/// Kind of an [`Error`]: the GenICam exception class pylon reports, which this crate also uses for
/// the failures it detects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    Generic,
    BadAlloc,
    InvalidArgument,
    OutOfRange,
    Property,
    Runtime,
    Logical,
    Access,
    Timeout,
    DynamicCast,
    /// Any other C++ exception.
    Unknown,
}

/// A failed pylon call or a call this crate rejects.
#[derive(Debug, Clone)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Error { kind, message: message.into() }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// The description of the failure, from pylon or from the checks of this crate.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

/// Turns a status into a result; the only place where statuses become errors.
fn check(status: sys::PylonStatus) -> Result<()> {
    use sys::PylonStatus as S;
    let kind = match status {
        S::PYLON_OK => return Ok(()),
        S::PYLON_ERROR_GENERIC => ErrorKind::Generic,
        S::PYLON_ERROR_BAD_ALLOC => ErrorKind::BadAlloc,
        S::PYLON_ERROR_INVALID_ARGUMENT => ErrorKind::InvalidArgument,
        S::PYLON_ERROR_OUT_OF_RANGE => ErrorKind::OutOfRange,
        S::PYLON_ERROR_PROPERTY => ErrorKind::Property,
        S::PYLON_ERROR_RUNTIME => ErrorKind::Runtime,
        S::PYLON_ERROR_LOGICAL => ErrorKind::Logical,
        S::PYLON_ERROR_ACCESS => ErrorKind::Access,
        S::PYLON_ERROR_TIMEOUT => ErrorKind::Timeout,
        S::PYLON_ERROR_DYNAMIC_CAST => ErrorKind::DynamicCast,
        _ => ErrorKind::Unknown,
    };
    // SAFETY: pylon_last_error returns a non-NULL C string that stays valid until the next failure
    // on this thread (pylon_shim.h); it is copied before any other call.
    let message = unsafe { CStr::from_ptr(sys::pylon_last_error()) };
    Err(Error::new(kind, message.to_string_lossy()))
}

/// Value a shim function writes to its out parameter.
fn out<T: Default>(call: impl FnOnce(&mut T) -> sys::PylonStatus) -> Result<T> {
    let mut value = T::default();
    check(call(&mut value))?;
    Ok(value)
}

/// Handle a shim function writes to its out parameter; None if it writes NULL.
fn optional_handle<T>(
    call: impl FnOnce(&mut *mut T) -> sys::PylonStatus,
) -> Result<Option<NonNull<T>>> {
    let mut raw = ptr::null_mut();
    check(call(&mut raw))?;
    Ok(NonNull::new(raw))
}

/// Handle a shim function writes to its out parameter.
///
/// # Safety
/// `call` writes a non-NULL pointer whenever it returns PYLON_OK.
unsafe fn handle<T>(call: impl FnOnce(&mut *mut T) -> sys::PylonStatus) -> Result<NonNull<T>> {
    // SAFETY: call returned PYLON_OK, so it wrote a non-NULL pointer (caller contract).
    optional_handle(call).map(|raw| unsafe { raw.unwrap_unchecked() })
}

/// Collects the handles a shim function passes to its callback.
///
/// # Safety
/// `ctx` is a `Vec<NonNull<T>>` that is not otherwise accessed during the call.
unsafe extern "C" fn push_handle<T>(ctx: *mut c_void, raw: *mut T) {
    // SAFETY: ctx is a Vec<NonNull<T>> not otherwise accessed during the call (caller contract).
    let found = unsafe { &mut *ctx.cast::<Vec<NonNull<T>>>() };
    // SAFETY: handles passed to callbacks are non-NULL (pylon_shim.h).
    found.push(unsafe { NonNull::new_unchecked(raw) });
}

/// The input string as C string; the only place where interior NULs are rejected.
fn c_string(text: &str) -> Result<CString> {
    CString::new(text).map_err(|e| Error::new(ErrorKind::InvalidArgument, e.to_string()))
}

/// The string a shim function passes to its string callback; empty if it passes none.
fn string(
    call: impl FnOnce(sys::PylonStringCallback, *mut c_void) -> sys::PylonStatus,
) -> Result<String> {
    Ok(strings(call)?.pop().unwrap_or_default())
}

/// Strings a shim function passes to its string callback.
fn strings(
    call: impl FnOnce(sys::PylonStringCallback, *mut c_void) -> sys::PylonStatus,
) -> Result<Vec<String>> {
    let mut texts = Vec::<String>::new();
    check(call(Some(push_string), (&raw mut texts).cast()))?;
    Ok(texts)
}

/// # Safety
/// `ctx` is the `Vec<String>` of [`strings`]; `data` covers `len` bytes during the call.
unsafe extern "C" fn push_string(ctx: *mut c_void, data: *const c_char, len: usize) {
    // SAFETY: ctx is the vector of `strings`, which is not otherwise accessed during the call.
    let texts = unsafe { &mut *ctx.cast::<Vec<String>>() };
    // SAFETY: the shim passes data valid during the call (pylon_shim.h Strings).
    let text = unsafe { bytes(data.cast(), len) };
    texts.push(String::from_utf8_lossy(text).into_owned());
}

/// Memory reported by the shim as slice; NULL or empty memory gives an empty slice.
///
/// # Safety
/// Unless `data` is NULL, it points to `len` bytes that stay valid and unchanged for `'a`.
unsafe fn bytes<'a>(data: *const c_void, len: usize) -> &'a [u8] {
    if data.is_null() || len == 0 {
        &[]
    } else {
        // SAFETY: data is non-NULL and covers len bytes for 'a (caller contract).
        unsafe { std::slice::from_raw_parts(data.cast(), len) }
    }
}
