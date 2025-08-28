# mpv-syncwatch

Synchronize video playback between your friends. Inspired by the browser extension [SyncWatch](https://github.com/Semro/syncwatch?tab=readme-ov-file).

## Installation

This plugin is compatible with Windows. Place syncwatch.dll into your scripts folder. Create a config file named `syncwatch.toml` with the info below.
```toml
enable_on_start = true
server_url = "wss://server.syncwatch.space"
name = ""
room_name = ""
```
I personally use and have tested this plugin with [mpv-hero](https://github.com/stax76/mpv-hero).

## Known issues

- When MPV is launched with --idle it will crash, I have no idea why this is happening so if you do please LMK.