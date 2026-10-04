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
nvmimic side by side under bash, zsh, sh and dash, and compares everything
they print, return, set and write. It runs 322 test scripts written for
nvmimic, plus the tests from nvm's own test suite that can be compared this
way (174 of them so far).

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

`self install` also asks to add a `nvmimic` command to `~/.local/bin`, and
the answer is yes unless you say no. That way you can type
`nvmimic self install` again later without hunting for the file you
downloaded. Pass `--no-link` to skip it. nvmimic never puts a program named
`nvm` on your `PATH`, because real nvm is a shell function and scripts check
for that.

`nvmimic self uninstall` puts nvm's scripts back and removes nvmimic. Add
`--dry-run` to either one to see what it would do without changing anything.

### Upgrading nvm itself

nvmimic swaps out nvm's scripts, so nvm's own upgrade steps collide with it.
Pick one of these:

- Run `nvmimic self uninstall`, upgrade nvm the usual way, then run
  `nvmimic self install` again.
- Or run nvm's install script as usual. It writes nvm's scripts back over
  nvmimic's, so your shells run real nvm again until you run
  `nvmimic self install` once more. nvm's installer leaves nvmimic's own
  folder (`$NVM_DIR/.nvmimic`) alone, so nothing is lost in between.

If you installed nvm with git, `git status` in `$NVM_DIR` shows `nvm.sh`,
`nvm-exec` and `bash_completion` as modified while nvmimic is installed.
That's expected. It also means nvm's manual upgrade (`git checkout` of a newer
version) stops with git's "local changes would be overwritten" error instead
of quietly replacing nvmimic. Use the first option above in that case.

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
  It starts a lot of small programs. Run one at a time. More on it below.
- `cargo xtask bench --check` times sourcing `nvm.sh` and fails if nvmimic gets
  slower than the budget in `bench/budget.toml`.
- `cargo xtask gen-shell` rebuilds `shell/nvm.sh` from nvm's own `nvm.sh`.

### How the parity tests stay quick

Running real nvm is the slow half of a comparison, and real nvm at one
version gives the same answer every time. So the first run saves what real
nvm did for each test script under each shell, and later runs read that back
instead of running nvm again. These saved answers are called recordings.
They live in `target/parity-golden/` and stay on your machine.

A recording is made again by itself when anything it depends on changes: the
test script, the copy of nvm in `tests/reference/`, the shell program, your
operating system version, or the test code that runs a script.

Some more things you can ask for:

- `cargo xtask parity` is the everyday run. Where real nvm did exactly the
  same thing under bash, zsh, sh and dash, the script runs under one of them,
  a different one each run. Where the shells differ, it runs under all four.
- `cargo xtask parity --all-shells` runs every script under every shell.
- `cargo xtask parity --changed` runs only the scripts your change can
  affect. Each script lists the nvm functions it runs in `# covers:` lines,
  and each Rust file names the nvm functions it rewrites in `nvm:` comments.
  A change to a file picks the scripts that cover its functions. A change
  this can't place runs everything.
- `cargo xtask parity --full` trusts no recording. It runs real nvm and
  nvmimic side by side on everything and tells you if a recording had gone
  out of date. Run it before a release and when moving to a new nvm version.
- `cargo xtask parity covers` works out the `# covers:` lines again by
  running real nvm with a note-taking line in each of its functions.

### A warning about macOS and the parity tests

macOS checks every new program the first time it runs. The check is called
Gatekeeper, and one system service does all of it, one file at a time. An
earlier version of the parity tests created and ran thousands of new scripts
in a few minutes. That service fell so far behind that it crashed, and the
whole computer stopped being able to start programs until it was rebooted.

The tests are built to avoid that now:

- Scripts that many tests run exist once, and each test gets a link to them.
- Nothing is deleted while the tests run. Leftovers in the temp folder
  (`nvmimic-diff-*` and similar) are cleaned up by a later run, once they're
  half an hour old.
- A guard runs a brand new script every two seconds and times it. If that
  gets slow, the tests pause until it's fast again. If it stays slow, the
  tests stop.

If you change how the tests create files, try one test first, then a small
group, and read the `gatekeeper guard:` line each run prints. You can also
turn the checks off for your terminal: System Settings, Privacy & Security,
Developer Tools, then switch your terminal app on and restart it.

## License

MIT, the same license as nvm. Parts of nvmimic are ported from nvm, so nvm's
copyright notice sits next to ours in [LICENSE](LICENSE).
