[![Build Actions Status](https://github.com/yantonov/alias/workflows/ci/badge.svg)](https://github.com/yantonov/alias/actions)

# Intro

**Git aliases for any command-line program.**

Inspired by [git aliases](https://git-scm.com/book/en/v2/Git-Basics-Git-Aliases), `alias` lets you define custom aliases, commands, and subcommands for any CLI — even if the program itself has no alias support.

Your aliases behave like built-in commands, without polluting your shell configuration or requiring separate wrapper scripts.

Technically, `alias` is just a thin wrapper around the target command-line application.

# Table of contents
1. [Technical notes](#technical-notes)
2. [Installation](#installation)
3. [Alias types](#alias-types)
4. [Alias groups and subcommands](#alias-groups-and-subcommands)
5. [List of aliases](#list-of-aliases)
6. [Dry run](#dry-run)
7. [Override](#override)
8. [Target executable location](#target-executable-location)
9. [Different operating systems](#different-operating-systems)
10. [Windows: run it from a POSIX shell](#windows-run-it-from-a-posix-shell)
11. [Shell scripts on Windows](#shell-scripts-on-windows)
12. [Examples](#examples)

## Technical notes
Technically, it is just a thin wrapper (proxy) that conditionally runs the target program.  
If an alias is found, it is expanded and the resolved version is used; otherwise, the target executable is called with the original arguments.

This app is independent of
1. the target program that needs alias support
2. the operating system
3. the shell/command interpreter

Configuration settings are stored in a separate config file,  
therefore, you do not need to pollute the global namespace with shell aliases (using .zshrc/.bashrc/.profile, etc.).

## Installation

### Manual
1. Put the executable in a directory on your PATH, and name it the same as the target program (the program without alias support)  
You can get prebuilt binaries [here](https://github.com/yantonov/alias/releases)
2. Write a config (config.toml) and put it next to the executable  
(a sample config will be created on the first launch if it does not exist)
3. Use custom aliases just as if they were supported out of the box.  

### Automatic
You can use this snippet to install the alias binary under a selected name into the ${HOME}/bin/<APP_NAME>-aliases directory, where <APP_NAME> is the name of the app that you want to configure
```bash
    curl -fsSL "https://raw.githubusercontent.com/yantonov/alias/master/bin/install/install.sh" | bash -s -- "<APP_NAME>"
```
The installer resolves the latest published release and takes everything from it: the scripts it runs, and the binary they download, which is verified against the checksum published beside it.  
A specific release can be installed with `ALIAS_VERSION`:
```bash
    curl -fsSL "https://raw.githubusercontent.com/yantonov/alias/master/bin/install/install.sh" | ALIAS_VERSION=0.2.7 bash -s -- "<APP_NAME>"
```
The line above still fetches the entry script itself from `master`. To pin that as well, replace `master` in the URL with a release tag.

## Alias types

**Regular alias** — expands to a sequence of arguments passed to the target program:
```toml
[alias]
co = "checkout main"
```
| Command | Expands to |
|---------|------------|
| `git co` | `git checkout main` |

Arguments are split the way git splits its own aliases: runs of whitespace separate arguments, `"..."` and `'...'` keep spaces inside a single argument, and a backslash takes the next character literally (except inside single quotes, where it is an ordinary character).  
TOML literal strings keep quoted aliases readable, with no escaping:
```toml
[alias]
ci = 'commit -m "work in progress"'
```
| Command | Arguments passed to the target |
|---------|--------------------------------|
| `git ci` | `commit`, `-m`, `work in progress` |

An unterminated quote or a trailing backslash is reported as a configuration error instead of being passed on to the target program.

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
Set `ALIAS_DRY_RUN` to see what a command expands to. Nothing is executed.

Given this config for a wrapper named `git`:
```toml
executable="/usr/bin/git"

[alias]
ci = 'commit -m "work in progress"'
today = "!git log --since=midnight --oneline"
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

A shell alias shows what the shell is handed. This follows the `sh -c COMMAND [NAME [ARGUMENT...]]` convention: `[2]` is the command that runs, with a `"$@"` appended to it so that the trailing arguments reach it, `[3]` becomes `$0` and only ever shows up in the shell's own error messages, and the rest are the positional parameters:
```
$ ALIAS_DRY_RUN=1 git today --author=you
dry run: ALIAS_DRY_RUN is set, nothing is executed
executable: /bin/sh
argv:
  [1] -c
  [2] git log --since=midnight --oneline "$@"
  [3] git log --since=midnight --oneline
  [4] --author=you
```

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

## Different operating systems
Different operating systems place binary files in different directories.  
To handle this, it is possible to reference the target executable using environment variables (example: executable="${HOME}/tools/bin/app")  
This helps you to use the same config file across different operating systems.

## Windows: run it from a POSIX shell
On Windows the app is meant to be used from a POSIX shell: Git Bash, MSYS2, Cygwin or WSL.  
It takes the shell to use from the `SHELL` environment variable, which PowerShell and cmd.exe do not set, so it exits with an error there.  
This is intentional: shell aliases are `sh` commands and would not survive being handed to PowerShell or cmd.exe anyway.

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
