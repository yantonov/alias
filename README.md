[![Build Status](https://travis-ci.com/yantonov/alias.svg?branch=master)](https://travis-ci.com/yantonov/alias)

This app helps you to define custom alias to command line utility which has no [alias support](https://git-scm.com/docs/git-config#Documentation/git-config.txt-alias).

Motivation: 
Let's suppose that command line utility has no alias support like git.  
Using this app you can define aliases (including shell aliases) and use it with any program.

Technically is just a thin wrapper to conditionally run target program.  

This app is independent from 
1. the target program which needs aliases support
2. operating system
3. command interpreter

Usage:
1. Put the executable to path, and name it the same as cli program without alias support
2. Write config (config.toml) and put it near the executable 
(sample config will be created if it does not exist)
3. Use custom aliases just like if they are supported out of the box. So you don't need to pollute global namespace with shell aliases (bashrc/.profile etc).

Sample config can be found [here](https://github.com/yantonov/alias/blob/master/docs/sample_config.toml).  
A little bit more realistic example - [arc aliases](https://github.com/yantonov/arc-aliases)  
It is possible to reference target executable using environment variables (example: executable="${HOME}/tools/bin/app")  
