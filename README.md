[![Build Actions Status](https://github.com/yantonov/alias/workflows/ci/badge.svg)](https://github.com/yantonov/alias/actions)

# Intro

This app helps you to define custom aliases/commands for a command-line utility that has no alias support,  
an example of functionality from [git](https://git-scm.com/book/en/v2/Git-Basics-Git-Aliases), another page about git config [section](https://git-scm.com/docs/git-config#Documentation/git-config.txt-alias)).  

Using this app you can define some aliases/commands/subcommands and use them just like they were defined out of the box.  

# Table of contents
1. [Technical notes](#technical-notes)
2. [Usage](#usage)
3. [Alias types](#alias-types)
4. [Alias groups and subcommands](#alias-groups-and-subcommands)
5. [List of aliases](#list-of-aliases)
6. [Override](#override)
7. [Target executable location](#target-executable-location)
8. [Different operating systems](#different-operating-systems)
9. [Shell scripts on Windows](#shell-scripts-on-windows)
10. [Examples](#examples)

## Technical notes
Technically is just a thin wrapper(proxy) to conditionally run target program.  
If alias is found, it is expanded and resolved version is used, otherwise target executable is called using original arguments.

This app is independent of
1. the target program which needs aliases support
2. operating system
3. shell/command interpreter

Configuration settings are stored in a separate config,  
therefore, you do not need to pollute a global namespace with shell aliases (using .zsh/.bashrc/.profile etc).

## Usage
1. Put the executable under PATH, and name it the same as the target program (program without alias support)  
You can get prebuilt binaries [here](https://github.com/yantonov/alias/releases)
2. Write config (config.toml) and put it near the executable  
(a sample config will be created at the first launch if it does not exist)
3. Use custom aliases just like if they are supported out of the box.  

## Alias types

**Regular alias** — expands to a sequence of arguments passed to the target program:
```toml
[alias]
co = "checkout main"
```
| Command | Expands to |
|---------|------------|
| `git co` | `git checkout main` |

**Shell alias** — prefixed with `!`, executed by the current shell:
```toml
[alias]
clean = "!rm -rf *.tmp"
```
| Command | Expands to |
|---------|------------|
| `git clean` | `rm -rf *.tmp` |

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
The list of aliases can be shown by using --aliases parameter.

## Override
You can add an additional configuration file 'override.toml' to the same directory.  
This helps you to redefine or introduce new aliases which depend on the environment.  
Motivation: some aliases may be specific to the working environment and you do not want to expose them by sharing using a public repository.

## Target executable location
There are two options:  
1. You can explicitly define the target executable using 'executable' parameter (see the example [here](https://github.com/yantonov/alias/blob/master/docs/sample_config.toml)).  
2. Without explicit configuration, the app tries to detect the target executable automatically by trying to find an existing file with the same name later in the PATH.  
In that case, you have to add this alias application in front of the target executable in terms of the PATH variable.

## Different operating systems
Different operating systems place binary files in different directories.  
To handle this, it is possible to reference target executable using environment variables (example: executable="${HOME}/tools/bin/app")  
This helps you to use the same config file across different operating systems.

## Shell scripts on Windows
When you try to use shell script directly as a target executable you can face the problem '%1 is not a valid win32 application'.  
To deal with this issue you can ann run_as_shell=true parameter to the config (or to the override file if you prefer), this will allows you to run the script using the current shell.

## Examples
Sample config can be found [here](https://github.com/yantonov/alias/blob/master/docs/sample_config.toml).

A little bit more realistic examples:  
1. [docker aliases](https://github.com/yantonov/docker-aliases)  
2. [podman aliases](https://github.com/yantonov/podman-aliases)  
3. [uv aliases](https://github.com/yantonov/uv-aliases)  
4. [gw aliases](https://github.com/yantonov/gw-aliases)  
5. [cdt aliases](https://github.com/yantonov/cdt-aliases)  
6. [arc aliases](https://github.com/yantonov/arc-aliases)  
7. [ya tool aliases](https://github.com/yantonov/ya-aliases)  
