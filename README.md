A screentime monitoring tool, shows how much time are you really using a PC. It work nicely with i3status or py3status on I3 window manager for example :)

# Install:
## Arch:
`yay screentime` (or other AUR manager)

## Locally:
`cargo install`

# Usage:
1. Run `screentime` in background to run the daemon
 2. Make requests
   -  `screentime get` will show you the total screentime in seconds
   -  `screentime hms` will show you the total screentime in HH:MM:SS format

# Config:

Config path: `$HOME/.config/screentime/config.toml`

Variables:

 - `port` (The port for the listening deamon (default `9898`)
 - `seconds_before_afk` (After how much seconds of inactivity assumes AFK (default `30`)
 - `snapshot_interval_in_seconds` (How often program saves its state on disk (default `10`)


## Features:

- [x] Linux, Windows and MacOS
- [x] Messure total screentime
- [x] AFK feature
- [x] Configurable
- [x] Simple client in a binary
