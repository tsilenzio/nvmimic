//! The `shelfy` example program of the nvmimic library, built here as well
//! so the harness's tests can run it as a second subject. All of it is in
//! `crates/nvmimic/examples/shelfy.rs`.

#[path = "../../../../crates/nvmimic/examples/shelfy.rs"]
mod program;

fn main() -> std::process::ExitCode {
    program::main()
}
