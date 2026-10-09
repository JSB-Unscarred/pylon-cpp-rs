//! Raw bindings to the pylon C shim. `shim/pylon_shim.h` documents every item; `bindings.rs` is
//! generated from it with `cargo build -p pylon-sys --features bindgen`.

include!("bindings.rs");
