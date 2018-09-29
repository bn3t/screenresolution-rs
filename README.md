
# MacOS screen resolution tool [![Build Status](https://travis-ci.org/bn3t/screenresolution-rs.svg?branch=master)](https://travis-ci.org/bn3t/screenresolution-rs)

## General Usage

```
$ cargo run -- -h
MacOS Screen Resolution Tool 0.1.1
Bernard Niset
Allows to list, get and set screen resolutions.

USAGE:
    screenresolution-rs <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    get     Get current active resution for current display
    help    Prints this message or the help of the given subcommand(s)
    list    List available resolutions for current display
    set     Set current active resolution for current display

Build: a841faf - 2018-09-17
```

## Getting the current resolution

```
$ cargo run -- get -h
Get current active resution for current display

USAGE:
    screenresolution-rs get [FLAGS]

FLAGS:
    -h, --help       Prints help information
    -l, --long       Shows more details on the current resolution
    -V, --version    Prints version informationv
```

Example:

```
    $ screenresolution-rs get
    Display 0: 2560x1600x32@0  - pixel 2560x1600x32@0  -        - 16:10
```

## Listing all available resolutions

```
$ cargo run -- list -h
List available resolutions for current display

USAGE:
    screenresolution-rs list [FLAGS]

FLAGS:
    -h, --help       Prints help information
    -l, --long       Shows more details on the displayed resolutions
    -V, --version    Prints version information
```

Example:

```
$ cargo run -- list
    Finished dev [unoptimized + debuginfo] target(s) in 0.09s
     Running `target/debug/screenresolution-rs list`
Display 0: 2880x1800x32@0  - pixel 2880x1800x32@0  -        - 16:10
Display 0: 2560x1600x32@0  - pixel 2560x1600x32@0  -        - 16:10
Display 0: 2048x1280x32@0  - pixel 2048x1280x32@0  -        - 16:10
Display 0: 1920x1200x32@0  - pixel 3840x2400x32@0  - HiDPI  - 16:10
Display 0: 1680x1050x32@0  - pixel 3360x2100x32@0  - HiDPI  - 16:10
Display 0: 1680x1050x32@0  - pixel 1680x1050x32@0  -        - 16:10
Display 0: 1440x900x32@0   - pixel 2880x1800x32@0  - HiDPI  - 16:10
Display 0: 1440x900x32@0   - pixel 1440x900x32@0   -        - 16:10
Display 0: 1280x800x32@0   - pixel 2560x1600x32@0  - HiDPI  - 16:10
Display 0: 1280x800x32@0   - pixel 1280x800x32@0   -        - 16:10
Display 0: 1024x768x32@0   - pixel 1024x768x32@0   -        - 4:3
Display 0: 1024x640x32@0   - pixel 2048x1280x32@0  - HiDPI  - 16:10
Display 0: 840x525x32@0    - pixel 1680x1050x32@0  - HiDPI  - 16:10
Display 0: 800x600x32@0    - pixel 800x600x32@0    -        - 4:3
Display 0: 720x450x32@0    - pixel 1440x900x32@0   - HiDPI  - 16:10
Display 0: 640x480x32@0    - pixel 640x480x32@0    -        - 4:3
```

## Setting a new screen resolution for a display

```
$ cargo run -- set -h
Set current active resolution for current display

USAGE:
    screenresolution-rs set [OPTIONS] <text-resolution|--interactive>

FLAGS:
    -h, --help           Prints help information
    -i, --interactive    Will allow to choose resolution interactively
    -V, --version        Prints version information

OPTIONS:
    -d, --display <DISPLAY>

ARGS:
    <RESOLUTION>    Resolution string in the form of WxHxP@R (e.g.: 1920x1200x32@0)
```

Example:

    $ cargo run -- set 2048x1280x32@0
