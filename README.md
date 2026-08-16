[![Build Actions Status](https://github.com/yantonov/alias/workflows/ci/badge.svg)](https://github.com/yantonov/alias/actions)

# Intro

**Git aliases for any command-line program.**

Inspired by [git aliases](https://git-scm.com/book/en/v2/Git-Basics-Git-Aliases), `alias` lets you define custom aliases, commands, and subcommands for any CLI — even if the program itself has no alias support.

Your aliases behave like built-in commands, without polluting your shell configuration or requiring separate wrapper scripts.

Technically, `alias` is just a thin wrapper around the target command-line application.

# Table of contents
1. [Installation](#installation)
2. [Alias types](#alias-types)
3. [Alias groups and subcommands](#alias-groups-and-subcommands)
4. [List of aliases](#list-of-aliases)
5. [Dry run](#dry-run)
6. [Override](#override)
7. [Target executable location](#target-executable-location)
8. [Endless loops](#endless-loops)
9. [Windows: shell aliases need a POSIX shell](#windows-shell-aliases-need-a-posix-shell)
10. [Shell scripts on Windows](#shell-scripts-on-windows)
11. [Examples](#examples)

## Installation

### Manual
1. Put the executable in a directory on your PATH, and name it the same as the target program (the program without alias support)  
You can get prebuilt binaries [here](https://github.com/yantonov/alias/releases)
2. Write a config (config.toml) and put it next to the executable  
(a sample config will be created on the first launch if it does not exist; in a directory that cannot be written to nothing is created, and `--aliases` says so)
3. Use custom aliases just as if they were supported out of the box.  

### Automatic
You can use this snippet to install the alias binary under a selected name into the ${HOME}/bin/<APP_NAME>-aliases directory, where <APP_NAME> is the name of the app that you want to configure
```bash
    curl -fsSL "https://raw.githubusercontent.com/yantonov/alias/master/bin/install/install.sh" | bash -s -- "<APP_NAME>"
```
The downloaded binary is verified against the checksum published beside it.  
A specific release can be installed with `ALIAS_VERSION`, where `<VERSION>` is the tag of a [published release](https://github.com/yantonov/alias/releases):
```bash
    curl -fsSL "https://raw.githubusercontent.com/yantonov/alias/master/bin/install/install.sh" | ALIAS_VERSION=<VERSION> bash -s -- "<APP_NAME>"
```
(the entry script itself still comes from `master`; replace it in the URL with the same tag to pin that as well)

## Alias types

**Regular alias** — expands to a sequence of arguments passed to the target program:
```toml
[alias]
co = "checkout main"
```
| Command | Expands to |
|---------|------------|
| `git co` | `git checkout main` |

Arguments are split the way git splits its own aliases, so `"..."` and `'...'` keep spaces inside a single argument.  
TOML literal strings keep quoted aliases readable, with no escaping:
```toml
[alias]
ci = 'commit -m "work in progress"'
```
| Command | Arguments passed to the target |
|---------|--------------------------------|
| `git ci` | `commit`, `-m`, `work in progress` |

**Shell alias** — prefixed with `!`, executed by the current shell:
```toml
[alias]
clean = "!rm -rf *.tmp"
```
| Command | Expands to |
|---------|------------|
| `git clean` | `rm -rf *.tmp` |

Whatever follows the alias is appended to the command, the way git appends it to its own shell aliases:
```toml
[alias]
tail = "!docker logs -f"
```
| Command | Runs |
|---------|------|
| `docker tail web` | `docker logs -f web` |

An argument keeps its spaces, so `docker tail "my container"` passes one argument, not two.  
Do not write `"$@"` in the alias yourself: it is added when there is something to pass, and writing it as well makes the arguments arrive twice.

## Alias groups and subcommands

Aliases can be organized into groups using TOML table nesting — or, from the user's perspective, you are defining **custom subcommands**. Both metaphors describe the same thing: a multi-word prefix that routes to a specific alias.

This is useful when a tool lacks a subcommand you want (`docker cleanup`, `git sync`, etc.) or when you want to extend an existing one. Groups allow you to use multi-word alias prefixes and can be nested to arbitrary depth.

**One-level group:**
```toml
[alias]
ps  = "container ls"
rmi = "image rm"
```
| Command | Expands to |
|---------|------------|
| `docker ps` | `docker container ls` |

**Nested groups:**
```toml
[alias.container]
clean = "!docker container prune -f"

[alias.image]
build = "image build -t" # group / subcommand
ls    = "image ls"

[alias.container.log]
tail = "!docker logs -f"     # doubly-nested group
```
| Command | Expands to |
|---------|------------|
| `docker container clean` | `docker container prune -f` |
| `docker image build myapp` | `docker image build -t myapp` |
| `docker container log tail` | `docker logs -f` |

## List of aliases
The list of aliases can be shown by using the --aliases parameter.

## Dry run
Set `ALIAS_DRY_RUN` to see what a command expands to. Nothing is executed. For a `!` alias it prints the shell invocation rather than the target's arguments.

Given this config for a wrapper named `git`:
```toml
executable="/usr/bin/git"

[alias]
ci = 'commit -m "work in progress"'
```

a regular alias shows the arguments the target program receives:
```
$ ALIAS_DRY_RUN=1 git ci --amend
dry run: ALIAS_DRY_RUN is set, nothing is executed
executable: /usr/bin/git
argv:
  [1] commit
  [2] -m
  [3] work in progress
  [4] --amend
```
`work in progress` is one argument, not three — which is the kind of thing there is otherwise no way to see.

Any value counts as set, and the variable is read on every run, so prefix a single command with it rather than exporting it: an exported one turns every wrapped tool into a no-op.

## Override
You can add an additional configuration file 'override.toml' to the same directory.  
This helps you to redefine existing aliases or introduce new ones that depend on the environment.  
Motivation: some aliases may be specific to the working environment, and you do not want to expose them by sharing them in a public repository.

## Target executable location
There are two options:  
1. You can explicitly define the target executable using the 'executable' parameter (see the example [here](https://github.com/yantonov/alias/blob/master/docs/sample_config.toml)).  
2. Without explicit configuration, the app tries to detect the target executable automatically by looking for an existing file with the same name later in the PATH.  
In that case, you have to place this alias application in front of the target executable in the PATH variable.

The 'executable' path can reference environment variables (example: executable="${HOME}/tools/bin/app"), which keeps one config file usable across operating systems that put binaries in different directories.

## Endless loops
A wrapper that ends up calling itself never stops. An `executable` entry pointing back at the wrapper, or at a symlink to it, is refused before anything runs. A loop that nothing tells apart from a working config — a shell alias invoking the alias it defines (`st = "!git st"` in a wrapper named `git`), or two wrappers naming each other — is bounded instead: the 16th nested call is refused.

The depth travels in `ALIAS_DEPTH`; setting it yourself only lowers that ceiling.

## Windows: shell aliases need a POSIX shell
Shell aliases are `sh` commands, and the shell to run them with is taken from the `SHELL` environment variable.  
On Windows that means a POSIX shell: Git Bash, MSYS2, Cygwin or WSL. PowerShell and cmd.exe do not set `SHELL`, and a shell alias invoked from there is reported as an error rather than handed to a shell it would not survive.  
Everything else has no use for a shell and works anywhere: regular aliases, groups, and any command that matches no alias and is forwarded to the target program.  
The same applies to any environment that leaves `SHELL` unset — a container, a systemd unit, a cron job, a CI step.

## Shell scripts on Windows
When you try to use a shell script directly as a target executable, you can face the problem '%1 is not a valid win32 application'.  
To deal with this issue, you can add the run_as_shell=true parameter to the config (or to the override file if you prefer); this will allow you to run the script using the current shell.

## Examples
Sample config can be found [here](https://github.com/yantonov/alias/blob/master/docs/sample_config.toml).

A few more realistic examples:  
1. [docker aliases](https://github.com/yantonov/docker-aliases)  
2. [podman aliases](https://github.com/yantonov/podman-aliases)  
3. [uv aliases](https://github.com/yantonov/uv-aliases)  
4. [cdt aliases](https://github.com/yantonov/cdt-aliases)  
5. [gw aliases](https://github.com/yantonov/gw-aliases)  
6. [arc aliases](https://github.com/yantonov/arc-aliases)  
7. [ya tool aliases](https://github.com/yantonov/ya-aliases)  
