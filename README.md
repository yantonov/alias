[![Build Actions Status](https://github.com/yantonov/alias/workflows/ci/badge.svg)](https://github.com/yantonov/alias/actions)

### Intro

This app helps you to define a custom alias for a command-line utility that has no [alias support](https://git-scm.com/docs/git-config#Documentation/git-config.txt-alias).

### Motivation
Let's suppose that some command-line utility has no alias support, like, for example, git.  
Using this app you can define some aliases (including shell aliases) and use them just like they were defined out of the box.
And, it is important, you do not want to pollute global namespace with shell aliases (using .zsh/.bashrc/.profile etc).

### Technical notes
Technically is just a thin wrapper to conditionally run target program.  

This app is independent of
1. the target program which needs aliases support
2. operating system
3. shell/command interpreter

### Usage
1. Put the executable to PATH, and name it the same as the target program (program without alias support)  
You can get prebuilt binaries [here](https://github.com/yantonov/alias/releases)
2. Write config (config.toml) and put it near the executable 
(sample config will be created if it does not exist)
3. Use custom aliases just like if they are supported out of the box.  

### About list of aliases
The list of aliases can be shown by using --aliases parameter.

### About overriding configuration
You can add an additional configuration file 'override.toml' to the same directory.  
This helps you to redefine or introduce new aliases which depend on the environment.  
Motivation: some aliases may be specific to the working environment and you do not want to expose them by sharing using a public repository.

### About target executable location
There are two options:  
1. You can explicitly define the target executable using 'executable' parameter (see sample [here](https://github.com/yantonov/alias/blob/master/docs/sample_config.toml)).  
2. Without explicit configuration, the app tries to detect the target executable automatically by trying to find an existing file with the same name later in the PATH.  
In that case, you have to add this alias application in front of the target executable in terms of the PATH variable.

### About different operating systems
Different operating systems place binary files in different directories.  
To handle this, it is possible to reference target executable using environment variables (example: executable="${HOME}/tools/bin/app")  
This helps you to use the same config file across different operating systems.

### About using shell scripts on Windows
When you try to use shell script directly as a target executable you can face the problem '%1 is not a valid win32 application'.  
To deal with this issue you can ann run_as_shell=true parameter to the config (or to the override file if you prefer), this will allows you to run the script using the current shell.

### Examples
Sample config can be found [here](https://github.com/yantonov/alias/blob/master/docs/sample_config.toml).

A little bit more realistic examples:  
1. [arc aliases](https://github.com/yantonov/arc-aliases)  
2. [docker aliases](https://github.com/yantonov/docker-aliases)  
3. [gw aliases](https://github.com/yantonov/gw-aliases)  
