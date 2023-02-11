/*
#[cfg(
    all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "ssse3"
    )
)]*/
mod flat_popcount;
mod traits;

pub use traits::RankSupport;
pub use flat_popcount::FlatPopcount;
