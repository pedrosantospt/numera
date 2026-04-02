//! Numera — A high-precision scientific calculator written in Rust.
//!
//! Library crate: all modules are declared here so `main.rs` (the binary)
//! and integration tests (`tests/`) can import them via `use numera::*`.
//!
//! # Quick Example
//!
//! ```
//! use numera::evaluator::Evaluator;
//! use numera::math::NumberFormat;
//!
//! let mut eval = Evaluator::new();
//! let (result, _fmt) = eval.evaluate("2 + 3", '.').unwrap();
//! assert_eq!(result.format_with(NumberFormat::General, 15, '.'), "5");
//! ```

pub mod constants;
pub mod evaluator;
pub mod functions;
pub mod gui;
pub mod history;
pub mod math;
pub mod settings;
pub mod tokenizer;
