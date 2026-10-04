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

Early work, copying nvm 0.40.8. Every nvm command now runs in nvmimic itself.
For downloading, checking and unpacking Node, and for npm, it starts the same
programs nvm starts (curl or wget, a checksum program, tar, npm). Opening a
shell and the everyday commands such as `nvm use`, `nvm ls` and `nvm current`
start no other program at all, with one small exception: the first
`nvm use` after Node or npm changes asks each of them for its version once
and remembers the answers.

How close it is gets measured, not guessed. A test harness runs real nvm and
nvmimic side by side under bash, zsh, sh and dash, and compares everything
they print, return, set and write. It runs 339 test scripts written for
nvmimic, plus the tests from nvm's own test suite that can be compared this
way (179 of them so far). Right now every one of them matches. Another 56
of nvm's tests install real Node releases from nodejs.org. Those run in a
separate pass that needs the internet. What still differs is listed under
[Known differences](#known-differences).

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

## Known differences

nvmimic aims to be impossible to tell apart from nvm. These are the places
where you still can, each with the reason. Nothing is left off this list on
purpose. If you find a difference that isn't here, it's a bug.

### Things only your own shell can see

Real nvm is shell code that runs inside your shell. nvmimic's `nvm` hands
the work to a separate program, and a separate program can't look inside
your shell. So these differ today:

- **You replace one of nvm's own functions.** If you define your own
  `nvm_ls`, real nvm calls yours from then on. nvmimic keeps using its own.
  nvm's test suite does this in 54 tests, which is why those aren't run.
- **Your install hook is a shell function.** `NVM_INSTALL_THIRD_PARTY_HOOK`
  can name a program or a function. nvmimic can only run a program.
- **A function or alias hides a tool.** If you define a function or alias
  called `node`, `npm`, `curl` or `grep`, for example, real nvm ends up
  running yours. nvmimic runs the real program on your `PATH`. Two cases
  already match: nvm skips a checksum program that is an alias, and
  `nvm debug` shows what each tool is aliased to.

There is a plan to close most of this: nvmimic would notice the change and
run that one command through nvm's own code inside your shell. It isn't
built yet. One part of it can't be done at all. The dash shell has no way to
show what a function contains, so under dash nvmimic can't tell that you
replaced one.

### The text of the functions

`declare -f nvm_ls` and `type nvm_ls` print the code of a function. Real nvm
shows its shell code there. nvmimic shows a one-line function that calls the
nvmimic program. They can't be the same text unless nvmimic is nvm.

For the same reason, `set -x` (which prints each command as it runs) shows
different lines inside an `nvm` call.

### Files nvmimic adds

- `$NVM_DIR/.nvmimic/` holds the nvmimic program, a list of what it
  replaced, the backups of nvm's scripts, and a small folder of remembered
  answers. Real nvm has no such folder.
- `nvm.sh`, `nvm-exec` and `bash_completion` in `$NVM_DIR` are nvmimic's
  versions, so their contents differ from nvm's.

### One extra command

`nvm __nvmimic ...` is how the shell function talks to the program. Real nvm
has no such command and prints its help for it.

### Node's and npm's versions are remembered

nvm finds out which Node is active by running `node --version`, and prints
npm's version after `nvm use` by running `npm --version`. It does both every
time. nvmimic asks once and reuses the answer until the file it asked
changes. For npm that also means npm's `package.json` and the `node`
program. Both are asked again when the `NODE_OPTIONS` setting changes. A
program that fails to answer is asked again every time, just like nvm does.

If something else ever changes what one of them prints, nvmimic would keep
showing the old answer. We tried broken npm settings files and didn't find
such a thing.

### Missing helper programs

nvm hands many small jobs to programs such as `sed`, `awk`, `sort` and
`uname`. nvmimic does most of those jobs itself. So if one of those programs
is missing from your `PATH`, or you put a different one in front, real nvm
fails or changes and nvmimic carries on as if nothing happened.

When a program is missing, the shell itself prints a complaint that names a
line in nvm's `nvm.sh`, such as
`nvm.sh: line 3740: npm: command not found`. nvmimic prints the same
complaint, with the same line number, in these places: `nvm debug`,
`nvm reinstall-packages`, the `uname` call that finds your operating system,
the `configure` and `make` steps of building from source, and
`nvm_rc_version` when it's called on its own. Everywhere else it stays quiet
where nvm's shell would complain.

Those complaints were measured with macOS's bash 3.2, zsh 5.9 and dash.
Another version of a shell, or a shell set to another language, may word
them differently.

### Output into a pipe nobody reads, under zsh

In `nvm install 20 | head -1`, `head` stops reading after one line. bash
and dash end nvm at its next line of output, and nvmimic ends the same way.
zsh lets nvm carry on, and each line nvm fails to print changes what it does
next. For example, it skips making the `default` alias. nvmimic under zsh
carries on too, but as if every line had printed, so it does make the alias.

### An nvm bug that deletes files

Under dash, a failed `nvm install` in nvm 0.40.8 deletes the folder your
`TMPDIR` setting points at. nvmimic doesn't copy bugs that destroy data
outside `$NVM_DIR`. Everything else about that failure matches.

### Not measured yet

These aren't known differences. They're places nobody has looked.

- **Linux.** Every test so far ran on macOS.
- **Building real Node from source.** The tests build from a stand-in source
  package with a tiny `configure` script and `Makefile`. Every step nvm
  takes around the build is compared. A real Node build, which takes most of
  an hour, isn't.
- **ksh.** nvm 0.40.8 itself doesn't run under macOS's ksh, so there's
  nothing to compare with.

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
- `cargo xtask parity --network` runs the tests from nvm's own suite that
  install real Node releases, which the everyday run leaves out. It
  downloads from nodejs.org, so it needs the internet. Each file is fetched
  once and kept in `target/parity-network/`. Try one test first:
  `NVMIMIC_CASES=NVT-installation_node/install_from_binary cargo xtask parity --network`.

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
