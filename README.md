# Portable Environment Initialization Scripts (for Windows)

Console application that generates scripts for initializing environment variables for different windows shells: batch, bash and powershell.

## Who needs this?

Are you a programmer working on Windows? Do you need to work with multiple toolchains/environment variables configurations in console? Do you use different shells available for windows: batch, bash, powershell? Do you have trouble maintaining multiple different environemt manipulation scripts? Don't want to pollute your %PATH% by putting everything in it? Suffer no more - this application is for you!

## How does the application work?

This is a simple console application:
```
Usage: portable_env [options]

Options:
    --config=FILE  Location of the config file. [default: ./portable_env.toml]
    --output=DIR  Where to put generated script directories. [default: .]
```

When run, the application generates 3 directories: `bash`, `cmd`, `ps`, each one contaning environment setup scripts for the respective shell. The directories contain scripts generated according to specifications in the config file.

Example specification:

```toml
[scripts]
ant = [
    {command = 'source', env = 'jdk17'},
    {command = 'env', key = 'ANT_HOME', value = 'C:\portable\ant-1.9', mode = 'PATH'},
    {command = 'env', key = 'PATH', value = '%ANT_HOME%\bin', mode = 'PREPEND_PATH'}
]
cmake = [
    {command = 'env', key = 'PATH', value = 'C:\Program Files (x86)\CMake 2.8\bin', mode = 'PREPEND_PATH'}
]
jdk17 = [
    {command = 'env', key = 'JAVA_HOME', value = 'C:\Program Files\Java\jdk17', mode = 'PATH'},
    {command = 'env', key = 'JRE_HOME', value = 'C:\Program Files\Java\jdk17\jre', mode = 'PATH'},
    {command = 'env', key = 'PATH', value = '%JAVA_HOME%\jre\bin', mode = 'PREPEND_PATH'},
    {command = 'env', key = 'PATH', value = '%JAVA_HOME%\bin', mode = 'PREPEND_PATH'}
]
jdk18 = [
    {command = 'env', key = 'JAVA_HOME', value = 'C:\Program Files\Java\jdk18', mode = 'PATH'},
    {command = 'env', key = 'JRE_HOME', value = 'C:\Program Files\Java\jdk18\jre', mode = 'PATH'},
    {command = 'env', key = 'PATH', value = '%JAVA_HOME%\jre\bin', mode = 'PREPEND_PATH'},
    {command = 'env', key = 'PATH', value = '%JAVA_HOME%\bin', mode = 'PREPEND_PATH'}
]
openssl32 = [
    {command = 'env', key = 'PATH', value = 'C:\OpenSSL-Win32\bin', mode = 'PREPEND_PATH'},
    # for mingw
    {command = 'env', key = 'CPATH', value = 'C:\OpenSSL-Win32\include', mode = 'PREPEND_PATH'},
    {command = 'env', key = 'LIBRARY_PATH', value = 'C:\OpenSSL-Win32\lib', mode = 'PREPEND_PATH'},
    {command = 'env', key = 'LIBRARY_PATH', value = 'C:\OpenSSL-Win32\lib\MinGW', mode = 'PREPEND_PATH'},
    #for vcpp
    {command = 'env', key = 'INCLUDE', value = 'C:\OpenSSL-Win32\include', mode = 'PREPEND_PATH'},
    {command = 'env', key = 'LIB', value = 'C:\OpenSSL-Win32\lib', mode = 'PREPEND_PATH'},
    {command = 'env', key = 'LIB', value = 'C:\OpenSSL-Win32\lib\VC', mode = 'PREPEND_PATH'}
]
putty = [
    {command = 'env', key = 'PATH', value = 'C:\portable\putty', mode = 'PREPEND_PATH'}
]
git = [
    {command = 'env', key = 'PATH', value = 'C:\Program Files\Git\cmd', mode = 'PREPEND_PATH'}
]
maven = [
    {command = 'source', env = 'jdk17'},
    {command = 'env', key = 'M2_HOME', value = 'C:\portable\maven', mode = 'PATH'},
    {command = 'env', key = 'PATH', value = '%M2_HOME%\bin', mode = 'PREPEND_PATH'}
]
mingw64 = [
    {command = 'env', key = 'MSYS_HOME', value = 'C:\portable\msys', mode = 'PATH'},
    {command = 'env', key = 'PATH', value = '%MSYS_HOME%/mingw64/bin;%MSYS_HOME%/usr/local/bin;%MSYS_HOME%/usr/bin;%MSYS_HOME%/bin', mode = 'PREPEND_PATH'}
]
msys = [
    {command = 'env', key = 'MSYS_HOME', value = 'C:\portable\msys', mode = 'PATH'},
    {command = 'env', key = 'PATH', value = '%MSYS_HOME%/usr/local/bin;%MSYS_HOME%/usr/bin;%MSYS_HOME%/bin', mode = 'PREPEND_PATH'}
]
```

Each [scripts] map entry is a script which will be converted to a `env_NAME.bat`/`env_NAME.ps1`/`env_NAME.sh` in the respective directory. A script is a list of commands, each described as a map:
```
{command = 'type', options...}
```
Currently there are 2 types of commands:

1. Source - sources `SCRIPTNAME` script from the map for convenient reuse
    ```
    {command = 'source', env = 'SCRIPTNAME'}
    ```

2. Env - modifies environment variable `ENV_KEY` using `ENV_VALUE`. 
    ```
    {command = 'env', key = 'ENV_KEY', value = 'ENV_VALUE', mode = 'MODE'}
    ```
    `ENV_VALUE` can contain windows env variable notation `%VARIABLE%`, which will be replaced with correct notation for each shell. `MODE` can have 4 values: 
    - `SET` (just sets the variable value), 
    - `PATH` (sets the variable value assuming it's a fs location, corrects for different path representations in shells), 
    - `PREPEND_PATH` (like path, but prepends to existing variable value), 
    - `APPEND_PATH` (like path, but appends to existing variable value).
    
## Using generated scripts

Add generated directories to PATH of your batch, bash and powershell shells. Then you can use generated scripts by simply sourcing them:

1. Bash: 
    ```
    . env_jdk17.sh
    ```
2. Batch:
    ```
    env_jdk17
    ```
3. Powershell:
    ```
    . env_jdk17.ps1
    ```
    
## Adding generated scripts to %PATH% of your shell

Your shell needs a way to find the generated scripts before you can use them. Here's how to configure that. Replace %OUTPUT_DIR% in the following guide with the directory in which you've generated the scripts.

### Batch

Just add `;%OUTPUT_DIR%/cmd` to your user %PATH% environment variable. Since you've found this project you probably already know how to do that. In case you don't, press [win] + Q, type env, [enter], click environment variables button and add an entry there.

### Powershell

Powershell has a default security policy which disables running scripts. You have enable running script files first, which is described in this guide: https://technet.microsoft.com/en-us/library/bb613481.aspx .

After that you need to set up your $env:PATH and $env:PATHEXT variable values for your shells. It's easiest to do that by creating a script file `%USERPROFILE%\Documents\WindowsPowerShell\profile.ps1` with the following contents:
```powershell
# allows your scripts to be found in path
$env:PATH="%OUTPUT_DIR%\ps;$env:PATH"
$env:PATHEXT=".PS1;$env:PATHEXT"
```

More info about powershell profiles can be found here:
https://technet.microsoft.com/en-us/library/bb613488(v=vs.85).aspx

### Bash

Exact details of setup for bash vary for each bash distribution. It's common however that bash shells execute `~/.bashrc` on startup. To add the generated scripts to the PATH variable you need to append the a variation of the following line to your `~/.bashrc`:
```bash
export PATH=`/%OUTPUT_DIR%/bash`:${PATH}
```

%OUTPUT_DIR% should be a proper unix path to the dir in which there are generated scripts.

## Development 

This app uses cargo as a build tool: https://crates.io/

## License

Copyright © 2016 Dariusz Antoniuk

Distributed under the GNU General Public License, version 3 or (at
your option) any later version.
