## ipmi.exe

```powershell
PS C:\Users\efika> ipmi.exe --help
ipmi 0.1.2
Hosts manager tool and ipmitool wrapper

USAGE:
    ipmi.exe [OPTIONS] [ipmitool-args]... [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -I <interface>        Override inteface <lanplus>
    -H <ip>               Override default if set on host IP
    -P <pswd>             Override default if set on host user password
    -U <user>             Override default if set on host user name

ARGS:
    <ipmitool-args>...    The ipmitool args to process

SUBCOMMANDS:
    help    Prints this message or the help of the given subcommand(s)
    host    Host management subcommand(s)
PS C:\Users\efika> # with the tool, goodbye:
PS C:\Users\efika> ipmitool.exe -I lanplus -H 000.000.000.000 -U admin -P admin <1st> <2nd> ...
PS C:\Users\efika> # say hello to:
PS C:\Users\efika> ipmi.exe <1st> <2nd> ...
PS C:\Users\efika> # [note]: please add ipmitool.exe to PATH before using it.
```

## Host management

### Host list example

```powershell
PS C:\Users\efika> # none in list
PS C:\Users\efika> ipmi.exe host list

---------------------------------------------
Index        IP                          User
-----        --                          ----
---------------------------------------------
Please add at least one host:
    ipmi.exe host add -i <ip> -u <user> -p <pswd>
And then use it:
    ipmi.exe host use <index of host>

PS C:\Users\efika> # some in list
PS C:\Users\efika> ipmi.exe host list

---------------------------------------------
Index        IP                          User
-----        --                          ----
 0001        10.245.38.1                ADMIN
 0002        10.245.38.1                admin
 0003        10.245.38.2                admin
 0004        10.245.38.3                 root
---------------------------------------------

PS C:\Users\efika>
```

### Host add example

```powershell
PS C:\Users\efika> ipmi.exe host list

---------------------------------------------
Index        IP                          User
-----        --                          ----
---------------------------------------------
Please add at least one host:
    ipmi.exe host add -i <ip> -u <user> -p <pswd>
And then use it:
    ipmi.exe host use <index of host>

PS C:\Users\efika> # add 1st host
PS C:\Users\efika> ipmi host add -i 10.245.38.1 -u ADMIN -p ADMIN
PS C:\Users\efika> ipmi.exe host list

---------------------------------------------
Index        IP                          User
-----        --                          ----
 0001        10.245.38.1                ADMIN
---------------------------------------------

PS C:\Users\efika> # add 2nd host
PS C:\Users\efika> ipmi host add -i 10.245.38.1 -u admin -p admin
PS C:\Users\efika> ipmi.exe host list

---------------------------------------------
Index        IP                          User
-----        --                          ----
 0001        10.245.38.1                ADMIN
 0002        10.245.38.1                admin
---------------------------------------------

PS C:\Users\efika> # add 3rd host
PS C:\Users\efika> ipmi host add -i 10.245.38.2 -u admin -p admin
PS C:\Users\efika> ipmi.exe host list

---------------------------------------------
Index        IP                          User
-----        --                          ----
 0001        10.245.38.1                ADMIN
 0002        10.245.38.1                admin
 0003        10.245.38.2                admin
---------------------------------------------

PS C:\Users\efika> # update 3rd host's password
PS C:\Users\efika> ipmi host add -i 10.245.38.2 -u admin -p AdMiN
PS C:\Users\efika> ipmi.exe host list

---------------------------------------------
Index        IP                          User
-----        --                          ----
 0001        10.245.38.1                ADMIN
 0002        10.245.38.1                admin
 0003        10.245.38.2                admin
---------------------------------------------

PS C:\Users\efika> # add 4th host
PS C:\Users\efika> ipmi host add -i 10.245.38.3 -u root -p root
PS C:\Users\efika> ipmi.exe host list

---------------------------------------------
Index        IP                          User
-----        --                          ----
 0001        10.245.38.1                ADMIN
 0002        10.245.38.1                admin
 0003        10.245.38.2                admin
 0004        10.245.38.3                 root
---------------------------------------------

PS C:\Users\efika>
```

### Host  use example

```powershell
PS C:\Users\efika> ipmi.exe host use 3
PS C:\Users\efika> ipmi.exe host list

---------------------------------------------
Index        IP                          User
-----        --                          ----
 0001        10.245.38.1                ADMIN
 0002        10.245.38.1                admin
*0003        10.245.38.2                admin
 0004        10.245.38.3                 root
---------------------------------------------

PS C:\Users\efika>
```

### Host del example

```powershell
PS C:\Users\efika> ipmi.exe host del 2
PS C:\Users\efika> ipmi.exe host list

---------------------------------------------
Index        IP                          User
-----        --                          ----
 0001        10.245.38.1                ADMIN
*0002        10.245.38.2                admin
 0003        10.245.38.3                 root
---------------------------------------------

PS C:\Users\efika>
```

## Ipmitool wrapper

```powershell
PS C:\Users\efika> # overall example of this tool, i hide part of ip with * for security:
PS C:\Users\efika> ipmi.exe host list

---------------------------------------------
Index        IP                          User
-----        --                          ----
---------------------------------------------
Please add at least one host:
    ipmi.exe host add -i <ip> -u <user> -p <pswd>
And then use it:
    ipmi.exe host use <index of host>

PS C:\Users\efika> ipmi.exe host add -i 10.245.38.*** -u ADMIN -p ADMIN
PS C:\Users\efika> ipmi.exe host list

---------------------------------------------
Index        IP                          User
-----        --                          ----
 0001        10.245.38.***              ADMIN
---------------------------------------------

PS C:\Users\efika> ipmi.exe host use 1
PS C:\Users\efika> ipmi.exe host list

---------------------------------------------
Index        IP                          User
-----        --                          ----
*0001        10.245.38.***              ADMIN
---------------------------------------------

PS C:\Users\efika> ipmi.exe user list
ID  Name             Callin  Link Auth  IPMI Msg   Channel Priv Limit
2   ADMIN            false   false      true       ADMINISTRATOR
3   Administrator    true    true       true       ADMINISTRATOR
PS C:\Users\efika>
```

