//! ADR-0005 enforcement: the no-raw-egress type wall, verified at compile time.
//!
//! `Gradient` must not be constructible from a `ThermalFrame`. trybuild compiles the UI test and
//! asserts it FAILS to compile. The expected error is captured in
//! `tests/ui/gradient_no_from_thermalframe.stderr`.

#[test]
fn gradient_cannot_be_built_from_a_thermal_frame() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/gradient_no_from_thermalframe.rs");
}
