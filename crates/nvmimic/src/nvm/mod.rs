//! Ports of nvm's helper functions, one module per group of related helpers.
//! Each ported function names the nvm function it follows in an `nvm:` line.

pub mod alias;
pub mod awk;
pub mod colors;
pub mod local;
pub mod npm;
pub mod nvmrc;
pub mod output;
pub mod paths;
pub mod posix_re;
pub mod terminfo;
pub mod version;
