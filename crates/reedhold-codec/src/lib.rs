//! Deterministic canonical binary encoding.

#![forbid(unsafe_code)]

mod reader;
mod writer;

pub use reader::Reader;
pub use writer::Writer;
