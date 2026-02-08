//! # Nemcss Extractor
//!
//! This is the extractor module of Nemcss. It is responsible for extracting CSS classes from
//! a given HTML file or string, and returning a set of unique CSS classes that are used in the
//! project.
mod extractor;

pub use extractor::extract_classes;
