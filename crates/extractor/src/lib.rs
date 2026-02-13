//! # Nemcss Extractor
//!
//! This is the extractor module of Nemcss. It is responsible for extracting CSS classes from
//! a given HTML file or string, and returning a set of unique CSS classes that are used in the
//! project.
mod extractor;

pub use extractor::{
    ASTRO_CLASS_LIST_REGEX, CLASS_ATTRIBUTE_REGEX, CLASS_UTILITY_REGEX, JSX_CLASS_EXPRESSION_REGEX,
    SVELTE_CLASS_BINDING_REGEX, VUE_CLASS_BINDING_REGEX, extract_classes,
};
