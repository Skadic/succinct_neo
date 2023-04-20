#![deny(rustdoc::broken_intra_doc_links)] // error if there are broken intra-doc links
#![deny(rustdoc::invalid_html_tags)] // no broken html in docs
#![deny(rustdoc::invalid_rust_codeblocks)] // code blocks should not be broken

pub mod bit_vec;
pub mod int_vec;
pub mod rank_select;
pub mod rolling_hash;
pub mod traits;

#[cfg(test)]
pub mod test {
    pub mod res {
        macro_rules! res {
            ($path:expr) => {
                include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/test/", $path))
            };
        }
        pub mod texts {
            pub const ALL: [&str; 3] = [ALL_A, DNA, EINSTEIN];

            pub const ALL_A: &str = res!("as.txt"); 
            pub const DNA: &str = res!("dna.txt"); 
            pub const EINSTEIN: &str = res!("einstein.txt"); 
        }
    }
}
