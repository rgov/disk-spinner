#[cfg_attr(not(feature = "shishua-cli"), allow(dead_code))]
mod cli;

#[cfg(feature = "shishua-cli")]
pub use cli::ShishuaCliGenerator;
