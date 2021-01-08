[![Build Actions Status](https://github.com/yantonov/alias/workflows/ci/badge.svg)](https://github.com/yantonov/alias/actions)

### Intro

This app helps you to define custom alias for a command line utility which has no [alias support](https://git-scm.com/docs/git-config#Documentation/git-config.txt-alias).

### Motivation
Let's suppose that some command line utility has no alias support, like, for example, git.  
Using this app you can define some aliases (including shell aliases) and use it just like they were defined out of the box.
And, it is important, you do not want to pollute global namespace with shell aliases (using .zsh/.bashrc/.profile etc).

### Technical notes
Technically is just a thin wrapper to conditionally run target program.  

This app is independent from 
1. the target program which needs aliases support
2. operating system
3. shell / command interpreter

### Usage
1. Put the executable to PATH, and name it the same as the target program (program without alias support)
2. Write config (config.toml) and put it near the executable 
(sample config will be created if it does not exist)
3. Use custom aliases just like if they are supported out of the box.  

### About list of aliases
List of aliases can be shown by using --aliases parameter.

### About overriding configuration
You can add additional configuration file 'override.toml' to the same directory.  
This helps you to redefine or introduce new aliases which are depend on the environment.  
Motivation: some aliases maybe speficic to the working environment and you do not want to expose it.

### About target executable location
There are two options:  
1. You can explicitly define target executable using 'executable' parameter (see sample [here](https://github.com/yantonov/alias/blob/master/docs/sample_config.toml)).  
2. Without explicit configuration, app tries to detect target executable automatically by trying to find exiting file with the same name later in the PATH.  
In that case you have to add this alias application in front of the target executable in terms of PATH variable.

### About different operating systems
Different operating systems places binary files to different directories.  
To handle this, it is possible to reference target executable using environment variables (example: executable="${HOME}/tools/bin/app")  
This helps you to use the same config file across different operating systems.

### About using shell scripts on Windows
When you try to use shell script directly as a target executable you can face a problem '%1 is not a valid win32 application'.  
To deal with this issue you can ann run_as_shell=true parameter to the config (or to the override file if you prefer), this will allows you to run script using current shell.

### Examples
Sample config can be found [here](https://github.com/yantonov/alias/blob/master/docs/sample_config.toml).

A little bit more realistic examples:  
1. [arc aliases](https://github.com/yantonov/arc-aliases)  
2. [docker aliases](https://github.com/yantonov/docker-aliases)  
3. [gw aliases](https://github.com/yantonov/gw-aliases)  
