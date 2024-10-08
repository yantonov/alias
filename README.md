[![Build Actions Status](https://github.com/yantonov/alias/workflows/ci/badge.svg)](https://github.com/yantonov/alias/actions)

# Intro

This app helps you to define a custom alias for a command-line utility that has no [alias support](https://git-scm.com/docs/git-config#Documentation/git-config.txt-alias).  
Using this app you can define some aliases (including shell aliases) and use them just like they were defined out of the box.  
Therefore, you do not need to pollute a global namespace with shell aliases (using .zsh/.bashrc/.profile etc).  

# Table of contents
1. [Technical notes](#technical-notes)
2. [Usage](#usage)
3. [List of aliases](#list-of-aliases)
4. [Override](#override)
5. [Target executable location](#target-executable-location)
6. [Different operating systems](#different-operating-systems)
7. [Shell scripts on Windows](#shell-scripts-on-windows)
8. [Examples](#examples)

## Technical notes
Technically is just a thin wrapper(proxy) to conditionally run target program.  

This app is independent of
1. the target program which needs aliases support
2. operating system
3. shell/command interpreter

## Usage
1. Put the executable under PATH, and name it the same as the target program (program without alias support)  
You can get prebuilt binaries [here](https://github.com/yantonov/alias/releases)
2. Write config (config.toml) and put it near the executable  
(a sample config will be created at the first launch if it does not exist)
3. Use custom aliases just like if they are supported out of the box.  

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
3. [gw aliases](https://github.com/yantonov/gw-aliases)  
4. [cdt aliases](https://github.com/yantonov/cdt-aliases)  
5. [arc aliases](https://github.com/yantonov/arc-aliases)  
6. [ya tool aliases](https://github.com/yantonov/ya-aliases)  
