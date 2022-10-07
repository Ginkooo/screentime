A minimal screentime deamon

# Install:
## Arch:
`yay screentime` (or other AUR manager)

## Locally:
`cargo install`

# Usage:
1. Run `screentime`
2. Make a `GET` request to `http://127.0.0.1:9898` and it will respond you with your today's screentime

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
- [ ] Simple client in a binary
