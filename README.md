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

This is nvmimic 0.1.0, an early release. It copies nvm 0.40.8 and runs on
macOS. It passes its own tests, but few people have used it yet, so expect
rough edges and keep `nvmimic self eject` in mind as the way back.

Every nvm command now runs in nvmimic itself.
For downloading and unpacking Node, and for npm, it starts the same programs
nvm starts (curl or wget, tar, npm). curl and wget stay on purpose: they
already know your proxy, your `~/.curlrc` and your system's certificates.
The checksum of a download is the one job nvmimic took over. It looks for
the same checksum program nvm would use and names it like nvm does, then
works out the SHA-256 sum itself. Opening a
shell and the everyday commands such as `nvm use`, `nvm ls` and `nvm current`
start no other program at all, with one small exception: the first
`nvm use` after Node or npm changes asks each of them for its version once
and remembers the answers.

How close it is gets measured, not guessed. A test harness runs official nvm and
nvmimic side by side under bash, zsh, sh and dash, and compares everything
they print, return, set and write. It runs 343 test scripts written for
nvmimic, plus the tests from nvm's own test suite that can be compared this
way (177 of them so far). Right now every one of them matches. Another 63
of nvm's tests install real Node releases from nodejs.org. Those run in a
separate pass that needs the internet. What still differs is listed under
[Known differences](#known-differences).

## Installing

On a Mac, [Homebrew](https://brew.sh) is the short way:

```sh
brew install tsilenzio/tap/nvmimic
nvmimic self install
```

Or download the program yourself. One file runs on both Apple silicon and
Intel Macs:

```sh
curl -LO https://github.com/tsilenzio/nvmimic/releases/latest/download/nvmimic-darwin-universal.tar.gz
tar -xzf nvmimic-darwin-universal.tar.gz
./nvmimic/nvmimic self install
```

There's no download for Linux yet. There, and anywhere else, build it with
Rust:

```sh
cargo build --release
./target/release/nvmimic self install
```

`self install` sets nvmimic up for `$NVM_DIR` (usually `~/.nvm`). You don't
need nvm first. With no nvm on the machine, nvmimic makes that folder and
you start from scratch. If nvm is already there, nvmimic takes it over: it keeps your Node versions, aliases and
settings, keeps a copy of nvm's own scripts in a backup folder, and offers to copy
the whole of `$NVM_DIR` somewhere safe first. If you installed nvm with
Homebrew, it asks before running `brew uninstall nvm`, because shells that
load Homebrew's copy would keep running official nvm.

You can stop `self install` at any moment, and a crash or a power cut can
too. It copies nvm's scripts and writes down where the copies are before it
replaces anything, so nvm's scripts are never lost. Run `nvmimic self
install` again to finish, or `nvmimic self uninstall` to go back.

Running `nvmimic self install` again is also how you repair the setup, for
example after nvm's own install script wrote its scripts back over
nvmimic's.

`self install` also copies the nvmimic program to `~/.local/bin/nvmimic`.
That way you can type `nvmimic self install` again later without hunting for
the file you downloaded. nvmimic never puts a program named `nvm` on your
`PATH`, because official nvm is a shell function and scripts check for that.

### Loading nvm in your shell

A shell gets the `nvm` command from a line in its startup file, such as
`~/.zshrc` or `~/.bashrc`. Two lines work.

The first is the line official nvm uses. If nvm was on the machine before,
it's already in your startup file and you change nothing:

```sh
export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"  # This loads nvm
```

Other tools look for `$NVM_DIR/nvm.sh` as well, and this line keeps working
when you go back to official nvm. Use it unless you have a reason not to.

The second asks the nvmimic program directly. Give it the name of the shell
the startup file belongs to: `bash`, `zsh`, `sh` or `dash`.

```sh
eval "$(nvmimic init zsh)"
```

Both lines load the same functions and switch to your default Node version.
A few things to know about the second one:

- `nvmimic` has to be on your `PATH` before that line runs.
- A wrong shell name does no harm. The code checks which shell is really
  running it. In any other shell it starts nvmimic a second time and ends
  up in the same place, without a warning.
- If your shell has no `NVM_DIR`, the nvm folder is the one `self install`
  set up. If there was no `self install`, it's the folder nvm's own
  installer picks: `$XDG_CONFIG_HOME/nvm` when `XDG_CONFIG_HOME` is set,
  and `~/.nvm` when it isn't.
- It loads nvmimic even after `nvmimic self eject`, and it fails once
  nvmimic is removed. Take the line out when you go back to official nvm.

Neither line loads tab completion. That has its own line, the same one as
with official nvm:

```sh
[ -s "$NVM_DIR/bash_completion" ] && \. "$NVM_DIR/bash_completion"
```

### Where nvmimic keeps its files

In `$NVM_DIR`, nvmimic only swaps nvm's three scripts (`nvm.sh`, `nvm-exec`
and `bash_completion`). Everything else of its own goes where many command
line tools keep theirs. Each place has a variable that moves it. Set it
before you run `self install`:

| What | Where | Variable |
|---|---|---|
| The nvmimic program | `~/.local/bin/nvmimic` | `NVMIMIC_INSTALL_PATH` |
| The list of what was installed | `~/.local/state/nvmimic` | `NVMIMIC_STATE_DIR` |
| The backups of nvm's scripts, and who answers `nvm` | `~/.local/state/nvm-handover` | `NVM_HANDOVER_DIR` |
| Remembered answers, safe to delete | `~/Library/Caches/nvmimic` on macOS, `~/.cache/nvmimic` elsewhere | `NVMIMIC_CACHE_DIR` |

The backups have a folder with no program's name on it because a sister
tool built on the same library can be set up for the same nvm folder, and
the two share them. nvmimic 0.1.0 kept the backups in
`~/.local/state/nvmimic/backup`. The first `self` command of a later
version moves them over.

If you use the `XDG_STATE_HOME` or `XDG_CACHE_HOME`
settings, nvmimic follows them too. If something that isn't nvmimic already
sits at `~/.local/bin/nvmimic`, `self install` stops and leaves it alone.

Add `--dry-run` to any `self` command to see what it would do without
changing anything. A dry run never asks a question. `--help` on a command
prints a short reminder of what it does and the flags it takes, and
`nvmimic self --help` lists the commands.

### Going back, and removing nvmimic

There are three ways out. All of them show what they're about to do and ask
first. `--force` skips the question. If there's no terminal to ask at, such
as inside a script, they change nothing unless you pass `--force`.

| Command | nvmimic's scripts | nvm's own scripts | The nvmimic program | Your Node versions and aliases |
|---|---|---|---|---|
| `nvmimic self eject` | removed | put back | stays | kept |
| `nvmimic self uninstall` | removed | put back, if nvmimic replaced any | removed | kept |
| `nvmimic self uninstall --purge` | removed | not put back | removed | deleted, with the whole nvm folder |

`self eject` hands the nvm folder back to nvm and keeps nvmimic around, so
`nvmimic self install` can take over again later. It needs an nvm to go back
to. If there was none before nvmimic, it says so and changes nothing.

`self uninstall` never deletes a Node version, an alias or the nvm folder,
even if nvmimic made that folder. If there was no nvm before nvmimic, you're
left with a folder of Node versions and no `nvm` command.

`--purge` is the one that deletes. It shows the folder and how big it is
before it asks.

If you changed one of nvmimic's scripts by hand, your version isn't thrown
away. It gets a new name, `nvm.sh.nvmimic-saved` for example, and nvmimic
tells you where it went.

If you ran nvm's own install script after nvmimic took over, the scripts in
the nvm folder are nvm's again, and likely newer than the ones nvmimic kept.
`self eject` and `self uninstall` leave those where they are and tell you
so. They don't put the older copies back over them.

Homebrew's nvm puts links into the nvm folder that point into Homebrew's
own folder, and `brew uninstall nvm` deletes what they point to. nvmimic
copies the scripts themselves before that happens, so `self eject` and
`self uninstall` give you working files back. A link whose target is still
there goes back as a link. A link that pointed at nothing before nvmimic
came along is never put back, and both commands tell you about it.

If Homebrew or cargo installed nvmimic, `self uninstall` leaves the program
alone and tells you the command that removes it. Run `nvmimic self
uninstall` before `brew uninstall nvmimic`. Homebrew runs nothing of
nvmimic's when it removes it, so nvmimic's `nvm.sh` would stay behind and
print a message about the missing program in every new shell.

### Updating nvmimic

`nvmimic self update` gets the newest nvmimic. Before anything changes, it
checks the download three ways: the checksum matches, nvmimic's developer
signed it, and it runs. Then it swaps the new program in and keeps the old
one. `nvmimic self rollback` brings the old one back.

If you installed nvmimic with Homebrew or cargo, update it with that tool
instead. `self update` tells you the command.

A copy you built yourself can't check who signed a download, so it refuses
to update itself. Build the newer version the same way you built this one.

### Upgrading nvm itself

nvmimic swaps out nvm's scripts, so nvm's own upgrade steps collide with it.
Pick one of these:

- Run `nvmimic self eject`, upgrade nvm the usual way, then run
  `nvmimic self install` again.
- Or run nvm's install script as usual. It writes nvm's scripts back over
  nvmimic's, so your shells run official nvm again until you run
  `nvmimic self install` once more. nvm's installer doesn't touch nvmimic's
  own files, so nothing is lost in between.

The zsh plugin zsh-nvm has an `nvm upgrade` command that does the second of
these for you. It runs git inside the nvm folder and puts nvm's scripts
back. Run `nvmimic self install` afterwards.

If you installed nvm with git, `git status` in `$NVM_DIR` shows `nvm.sh`,
`nvm-exec` and `bash_completion` as modified while nvmimic is installed.
That's expected. It also means nvm's manual upgrade (`git checkout` of a newer
version) stops with git's "local changes would be overwritten" error instead
of quietly replacing nvmimic. Use the first option above in that case.

## How it stays fast

nvmimic's `nvm.sh` is a few lines long. It starts the nvmimic program once,
as `nvmimic init`. That prints the same functions nvm defines, followed by
what `nvm use default` would change, and the shell takes all of it in with
`eval`.

One thing can cost a second start. A program only sees the variables your
shell exported, and nvm also reads some that are set without `export`. So
the printed code first checks that every variable nvmimic looked at holds
what nvmimic saw. If one doesn't, the shell starts nvmimic again and hands
every variable over. You get the same result either way. Official nvm starts around 40
small programs for the same job, plus `node` in some cases.

On the test machine, `cargo xtask bench` measured the extra time a new shell
spends on nvm:

| Shell | Official nvm 0.40.8 | nvmimic |
|---|---|---|
| bash 3.2 | 144ms | 5ms |
| zsh | 346ms | 6ms |
| dash | 110ms | 4ms |

## Known differences

nvmimic matches what nvm's commands and documented helpers print, return,
set and write. That's the promise. A helper is one of the small functions
nvm's own README tells you to call, such as `nvm_find_nvmrc`.

It promises nothing if you replace nvm's own functions, give the tools nvm
runs an alias or a function of the same name, or remove programs nvm needs.

### Things only your own shell can see

This part is outside the promise.

Official nvm is shell code that runs inside your shell. nvmimic's `nvm`
hands the work to a separate program, and a separate program can't look
inside your shell. So nvmimic doesn't follow these:

- **Your own version of one of nvm's functions.** If you define your own
  `nvm_ls`, official nvm calls yours from then on. nvmimic's program never
  sees it. Calling `nvm_ls` yourself still runs yours.
- **A shell function as the install hook.** `NVM_INSTALL_THIRD_PARTY_HOOK`
  has to name a program for nvmimic to run it.
- **A shell function or alias with the name of a tool nvm runs.** If you
  define a function called `node`, `npm` or `curl`, official nvm ends up
  running yours. nvmimic runs the program. Two cases match anyway: nvm
  skips a checksum program that is an alias, and `nvm debug` shows what
  each tool is aliased to.

If you need any of these, you need official nvm. `nvmimic self eject` puts
its scripts back and keeps nvmimic for later, and `nvmimic self uninstall`
puts them back and removes nvmimic.

### The text of the functions

`declare -f nvm_ls` and `type nvm_ls` print the code of a function. Official nvm
shows its shell code there. nvmimic shows a one-line function that calls the
nvmimic program. They can't be the same text unless nvmimic is nvm.

For the same reason, `set -x` (which prints each command as it runs) shows
different lines inside an `nvm` call.

A few small functions are copied from nvm unchanged. Their text matches, but
they sit on other lines in nvmimic's `nvm.sh`, so an error message that
names a line number inside one of them names a different number.

### Files nvmimic adds

- `nvm.sh`, `nvm-exec` and `bash_completion` in `$NVM_DIR` are nvmimic's
  versions, so their contents differ from nvm's. Nothing else in `$NVM_DIR`
  is nvmimic's.
- Outside `$NVM_DIR`, nvmimic keeps the files listed under
  [Where nvmimic keeps its files](#where-nvmimic-keeps-its-files). Official nvm
  has none of them.

### One extra function

Loading nvmimic's `nvm.sh` defines every function nvm defines, plus one of
its own: `__nvmimic_handoff`. It's how `nvm` and the other functions pass
their work to the nvmimic program. You can see it in a list of all functions
(`declare -F` in bash). `nvm unload` removes it along with nvm's.

The functions carry nvmimic's version. If you update nvmimic while a shell
is open, that shell gets the new functions the next time you run `nvm`.

Nothing extra hides behind the name `nvm` itself. `nvm` accepts exactly the
commands official nvm accepts and prints nvm's help for anything else.

### Tab completion suggests more

nvm's `bash_completion` suggests commands, and versions and aliases after
some of them. nvmimic's does that and adds to it, on purpose:

- **Flags.** Type a dash and press TAB, as in `nvm install --`, and you get
  the flags that command takes. A flag that wants a value, such as
  `--alias=`, is filled in with no space after it, so you can type the
  value straight away. macOS's own bash 3.2 can't leave that space out, so
  there you delete it yourself.
- **Arguments in the right place.** `nvm use --silent ` still suggests
  versions, and `nvm cache ` suggests `dir` and `clear`. nvm suggests its
  command names in spots like `nvm install `, where one can't go. nvmimic
  doesn't.
- **One more command.** `set-colors` is in the list.
- **The `nvmimic` command.** `nvmimic self ` and its flags complete too.
- **A few words about each entry under zsh**, when zsh's own completion
  system is running.

It never goes to the network, so `nvm install ` doesn't list the versions
you could install. Each TAB starts the nvmimic program once.

In zsh, `complete -p nvm` prints nothing for nvmimic, because nvm is
registered with zsh's completion system directly.

### Node's and npm's versions are remembered

nvm finds out which Node is active by running `node --version`, and prints
npm's version after `nvm use` by running `npm --version`. It does both every
time. nvmimic asks once and saves the answer.

The rule: a saved answer is reused only when everything we know it depends
on is unchanged. If nvmimic can't be sure, it runs the real program, the
same as nvm. There's no setting to switch this off.

What gets checked before a saved answer is reused:

- the program's file: its full path, size, both timestamps and its inode
  (the number the disk gives each file)
- for npm, its `package.json` too, and the `node` program that runs it
- the `NODE_OPTIONS` setting
- the build of your operating system, so after a system update everything
  is asked again

Some things are never saved:

- A `node` that isn't a compiled program named `node` or `iojs`. Tools like
  asdf, Volta and mise put a small stand-in called a shim on your `PATH`
  under the name `node`. A shim picks a different Node depending on the
  folder you're in, so its answer is only good for one call.
- An `npm` that doesn't live inside a package named `npm`, or one whose
  `node` is such a shim.
- An answer from a program that failed. It's asked again every time, just
  like nvm does.

The limit: nvmimic can only check the things on this list. If something
else changes what `node --version` or `npm --version` prints, nvmimic keeps
showing the old answer. We tried broken npm settings files and didn't find
such a thing, but we can't prove there isn't one.

If you ever suspect a stale answer, delete the cache folder from
[Where nvmimic keeps its files](#where-nvmimic-keeps-its-files). The next
call asks the real programs again.

### Missing helper programs

nvm hands many small jobs to programs such as `sed`, `awk`, `sort` and
`uname`. nvmimic does most of those jobs itself. So if one of those programs
is missing from your `PATH`, or you put a different one in front, official nvm
fails or changes and nvmimic carries on as if nothing happened. This is
outside the promise too.

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

- **Linux.** The tests also run on Linux, in a container (Debian 12 on
  arm64), and all 2028 comparisons there match. One of nvm's own tests is
  left out on Linux. It defines a shell function called `manpath` on a
  machine with no `manpath` program, and `nvm use` then leaves `MANPATH`
  alone where nvm sets it. Everything else is tested on macOS, and there's
  no Linux download of nvmimic yet.
- **Building real Node from source.** The tests build from a stand-in source
  package with a tiny `configure` script and `Makefile`. Every step nvm
  takes around the build is compared. A real Node build, which takes most of
  an hour, isn't.
- **ksh.** nvm 0.40.8 itself doesn't run under macOS's ksh, so there's
  nothing to compare with.

## Building on nvmimic

If you want to use nvmimic from your own program, use the Rust library in
`crates/nvmimic`. That's the supported way. The `nvmimic` command line tool
is a thin layer over it.

The library doesn't decide where Node versions are kept. It asks a
`Backend` for that. A `Backend` is a Rust trait, which is a list of
questions a type has to answer: which versions are installed, where one of
them lives, how to remove one. nvmimic's own backend answers with nvm's
folders. If your program keeps Node somewhere else, write your own backend
and every nvm command works on top of it.

A backend can do more than answer those questions:

- install versions its own way (`install`)
- hear about it when a shell switches to a version or away from one
  (`switched`), and set a variable of its own in that shell
- keep downloads and the list of default packages in its own place
  (`cache_dir`, `default_packages_file`)
- say what it can't do (`offers`). Building from source is an example. Tab
  completion then leaves those entries out, and running one prints a
  message and stops.

The library doesn't decide what your program is called either. It takes an
`Identity`: a name, a version and a few other facts. Everything nvmimic
names after itself is then named after your program: its folders
(`~/.local/state/<name>`), its settings (`<NAME>_STATE_DIR`), its shell
function (`__<name>_handoff`) and the name in every message.

Three more pieces finish a program:

- `Host` answers the shell functions and prints what `<name> init` prints.
- `self_install_with`, `self_eject_with` and `self_uninstall_with` set your
  program up and remove it, with room for steps of your own (`Steps`).
- `self_disable_with` and `self_enable_with` put your program to sleep and
  wake it, for when someone has two of these programs.
- `Manifest`, the record of an install, keeps lines of your own.

`crates/nvmimic/examples/shelfy.rs` is a whole program built this way, in
about a hundred lines. It keeps Node on a "shelf" outside the nvm folder
and calls itself `shelfy`. `crates/nvmimic/tests/other_store.rs` runs it,
in real shells too.

Two programs built on the library can be set up for the same nvm folder.
Only one answers `nvm` at a time, and the other stays installed and asleep.
Whichever was told to take over last is in control:

- `self install` on the sleeping one says what will change and asks first.
  `--force` skips the question.
- `self uninstall` on the one in control hands `nvm` to the other. With no
  other program set up, official nvm's scripts go back.
- `self eject` always goes back to official nvm. A program that ejected or
  disabled itself stays asleep until it is told to take over again.

They keep track of each other in one small record,
`~/.local/state/nvm-handover/record`, next to the backups of official nvm's
scripts. It belongs to neither program, so removing one never takes the
backups away from the other. `crates/nvmimic-cli/tests/sister.rs` runs
nvmimic and `shelfy` through every one of these moves.

The parity tests can compare your program with official nvm as well. You
describe it as a `Subject` (in `tests/differential/src/subject.rs`): how it
gets installed, which shell functions are its own, and where it differs on
purpose. `tests/differential/tests/second_subject.rs` does that for
`shelfy`.

`nvm.sh` talks to the nvmimic program through the `__nvmimic_handoff`
function and an internal command of the program that isn't in its help.
Both are private. They're unsupported and may change in any release, so
don't call them from your own scripts.

## Working on it

- `cargo test --workspace` runs the unit tests.
- `cargo xtask parity` compares nvmimic with official nvm on every test script.
  It starts a lot of small programs. Run one at a time. More on it below.
- `cargo xtask bench --check` times sourcing `nvm.sh` and fails if nvmimic gets
  slower than the budget in `bench/budget.toml`.
- `cargo xtask smoke` installs nvmimic into a throwaway home folder, installs
  three real Node releases, and switches between them under each shell. It
  takes well under a minute once the downloads are kept.
- `cargo xtask gen-shell` rebuilds `shell/nvm.sh` and `shell/init.sh` from nvm's own `nvm.sh`.
- `cargo xtask release` builds the macOS program for a release into
  `target/dist/`: one file for both Apple silicon and Intel, signed and
  checked by Apple, packed with the license, plus checksums and a Homebrew
  formula. It needs a signing certificate. `--unsigned` skips that part so
  you can try the rest.

### How the parity tests stay quick

Running official nvm is the slow half of a comparison, and official nvm at one
version gives the same answer every time. So the first run saves what official
nvm did for each test script under each shell, and later runs read that back
instead of running nvm again. These saved answers are called recordings.
They live in `target/parity-golden/` and stay on your machine.

A recording is made again by itself when anything it depends on changes: the
test script, the copy of nvm in `tests/reference/`, the shell program, your
operating system version, or the test code that runs a script.

Some more things you can ask for:

- `cargo xtask parity` is the everyday run. Where official nvm did exactly the
  same thing under bash, zsh, sh and dash, the script runs under one of them,
  a different one each run. Where the shells differ, it runs under all four.
- `cargo xtask parity --all-shells` runs every script under every shell.
- `cargo xtask parity --changed` runs only the scripts your change can
  affect. Each script lists the nvm functions it runs in `# covers:` lines,
  and each Rust file names the nvm functions it rewrites in `nvm:` comments.
  A change to a file picks the scripts that cover its functions. A change
  this can't place runs everything.
- `cargo xtask parity --full` trusts no recording. It runs official nvm and
  nvmimic side by side on everything and tells you if a recording had gone
  out of date. Run it before a release and when moving to a new nvm version.
- `cargo xtask parity covers` works out the `# covers:` lines again by
  running official nvm with a note-taking line in each of its functions.
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
