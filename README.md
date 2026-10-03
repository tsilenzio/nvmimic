# nvmimic

> [!IMPORTANT]
> **If you can use [mise](https://mise.jdx.dev), use mise instead.** It manages
> Node and many other tools, and it's the better choice. nvmimic is only for
> places that need nvm itself, such as scripts or setups that call `nvm`, but
> want new shells to start much faster.
>
> nvmimic isn't affiliated with nvm-sh, the nvm project, or the OpenJS
> Foundation.

nvmimic is a copy of [nvm](https://github.com/nvm-sh/nvm) written in Rust. nvm
is the tool many people use to install Node and switch between Node versions.
nvmimic aims to behave exactly like nvm, so a script that uses nvm can't tell
the difference. Same commands, same output, same exit codes, same files on disk.

## Why it exists

nvm is a big shell script. Your shell runs it every time a new terminal opens,
and that takes a noticeable amount of time. On one test machine, opening a
shell with nvm took about 170ms in bash and about 400ms in zsh. Most of that
time goes into starting dozens of small helper programs.

nvmimic keeps only a tiny piece in your shell. That piece passes your `nvm`
commands to one fast program, so a new shell barely notices it.

## Status

Early work, copying nvm 0.40.8. Opening a shell, `nvm use`, `nvm ls`,
`nvm current`, `nvm which`, `nvm alias` and the other everyday commands run in
nvmimic itself. Commands that aren't ported yet, such as `nvm install`, still
work: nvmimic runs them with the real nvm 0.40.8 it carries inside, which takes
about as long as nvm does.

How close it is gets measured, not guessed. A test harness runs real nvm and
nvmimic side by side on 316 test scripts under bash, zsh, sh and dash, and
compares everything they print, return, set and write.

## Installing

There are no release downloads yet, so build it with Rust first:

```sh
cargo build --release
./target/release/nvmimic self install
```

`self install` puts nvmimic into `$NVM_DIR` (usually `~/.nvm`). If nvm is
already there, nvmimic takes it over: it keeps your Node versions, aliases and
settings, moves nvm's own scripts into `$NVM_DIR/.nvmimic/backup`, and offers
to copy the whole folder somewhere safe first. If you installed nvm with
Homebrew, it asks before running `brew uninstall nvm`, because shells that
load Homebrew's copy would keep running the real nvm.

`nvmimic self uninstall` puts nvm's scripts back and removes nvmimic. Add
`--dry-run` to either one to see what it would do without changing anything.

## How it stays fast

Sourcing nvmimic's `nvm.sh` defines the same functions nvm defines and then
starts one program, nvmimic itself, which works out what `nvm use default`
would do and hands the result back to the shell. Real nvm starts around 40
small programs for the same job, plus `node` in some cases.

On the test machine, `cargo xtask bench` measured the extra time a new shell
spends on nvm:

| Shell | Real nvm 0.40.8 | nvmimic |
|---|---|---|
| bash 3.2 | 158ms | 5ms |
| zsh | 359ms | 5ms |
| dash | 110ms | 3ms |

## Working on it

- `cargo test --workspace` runs the unit tests.
- `cargo xtask parity` compares nvmimic with real nvm on every test script.
- `cargo xtask bench --check` times sourcing `nvm.sh` and fails if nvmimic gets
  slower than the budget in `bench/budget.toml`.
- `cargo xtask gen-shell` rebuilds `shell/nvm.sh` from nvm's own `nvm.sh`.

## License

MIT, the same license as nvm. Parts of nvmimic are ported from nvm, so nvm's
copyright notice sits next to ours in [LICENSE](LICENSE).
