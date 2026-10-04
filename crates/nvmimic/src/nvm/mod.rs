//! Ports of nvm's helper functions, one module per group of related helpers.
//! Each ported function names the nvm function it follows in an `nvm:` line.

pub mod alias;
pub mod artifact;
pub mod awk;
pub mod checksum;
pub mod colors;
pub mod local;
pub mod lock;
pub mod npm;
pub mod nvmrc;
pub mod output;
pub mod paths;
pub mod platform;
pub mod posix_re;
pub mod remote;
pub mod terminfo;
pub mod tools;
pub mod version;
