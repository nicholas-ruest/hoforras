// ADR-0005 compile-fail guard: a `ThermalFrame` must NOT be convertible into a `Gradient`.
// If someone adds `impl From<ThermalFrame> for Gradient`, this file would start compiling and the
// trybuild test in `tests/compile_fail.rs` would fail — surfacing the no-raw-egress regression.

use hoforras_domain::{Gradient, ThermalFrame};

fn illegal_conversion(frame: ThermalFrame) -> Gradient {
    frame.into()
}

fn main() {
    let _ = illegal_conversion;
}
