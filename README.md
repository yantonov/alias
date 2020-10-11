[![Build Status](https://travis-ci.com/yantonov/alias.svg?branch=master)](https://travis-ci.com/yantonov/alias)

### Intro

This app helps you to define custom alias for a command line utility which has no [alias support](https://git-scm.com/docs/git-config#Documentation/git-config.txt-alias).

### Motivation
Let's suppose that some command line utility has no alias support, like, for example, git.  
Using this app you can define some aliases (including shell aliases) and use it just like they were defined out of the box.

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
So you don't need to pollute global namespace with shell aliases (.zsh/.bashrc/.profile etc).
4. List of aliases can be get using --aliases parameter.

### About different operating systems
Different operating systems places binary files to different directories.  
To handle this, it is possible to reference target executable using environment variables (example: executable="${HOME}/tools/bin/app")  
This helps you to use the same config file across different operating systems.

### Examples
Sample config can be found [here](https://github.com/yantonov/alias/blob/master/docs/sample_config.toml).

A little bit more realistic examples:  
1. [arc aliases](https://github.com/yantonov/arc-aliases)  
1. [docker aliases](https://github.com/yantonov/docker-aliases)  
