#![deny(rustdoc::broken_intra_doc_links)] // error if there are broken intra-doc links
#![deny(rustdoc::invalid_html_tags)] // no broken html in docs
#![deny(rustdoc::invalid_rust_codeblocks)] // code blocks should not be broken

pub mod bit_vec;
pub mod int_vec;
pub mod rank_select;
pub mod rolling_hash;
pub mod traits;
