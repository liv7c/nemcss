//! # Nemcss Scanner
//!
//! This is the scanner module of Nemcss. It is responsible for scanning the content files and
//! determining which CSS classes are used in the project. It will enable other parts of Nemcss to
//! only generate CSS classes that are actually used in the project, thus reducing the size of the
//! final CSS output.

mod extractor;
