//! Reference in-process adapters for the Node Isolation & Security ports (DDD-04).
//!
//! The real `rvm-*` substrate (rvm-coherence/-kernel/-witness/-proof/-security/-cap) is not
//! available in this environment (not published, not vendored). Per ADR-0001 the domain units
//! depend only on the ports, so these reference adapters can be swapped for the real RVM crates
//! later with no change to `coherence`/`capability`/`audit`. They run **in-process** (ADR-0004).

pub mod rvm_cap;
pub mod rvm_coherence;
pub mod rvm_witness;

pub use rvm_cap::RvmCapAdapter;
pub use rvm_coherence::RvmCoherenceAdapter;
pub use rvm_witness::RvmWitnessAdapter;
