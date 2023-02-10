/*
#[cfg(
    all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "ssse3"
    )
)]*/
pub mod flat_popcount;
mod traits;
