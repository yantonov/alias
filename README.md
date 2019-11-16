This app helps you to define custom alias to command line utility which has no [alias support](https://git-scm.com/docs/git-config#Documentation/git-config.txt-alias).

Motivation: 
Let's suppose that command line utility has no alias support like git.  
Using this app you can define aliases (including shell aliases) and use it with any program.

Technically is just a thin wrapper to conditionally run target program.  

This app is independent from 
1. the target program which needs for aliases support
2. operating system
3. command interpreter

Usage:
1. Put the executable to path, and name it the same as cli program without alias support
2. Write config (config.toml) and put it near the executable
3. Use custom aliases

Sample config can be found [here](https://github.com/yantonov/aliases_experimental/raw/master/sample_config.toml).
