# mpv-syncwatch

Synchronize video playback between your friends. Inspired by the browser extension [SyncWatch](https://github.com/Semro/syncwatch?tab=readme-ov-file).

## Installation


### Windows

- Place syncwatch.dll into your scripts folder, mine is in `<MPV_FOLDER>/portable_config/scripts`
- Create a config file called `syncwatch.toml` in `<MPV_FOLDER>/portable_config` and fill it out with the values below

### Linux

- Place libsyncwatch.so into your scripts folder, usually located in `~/.config/mpv/scripts`
- Create a config file called `syncwatch.toml` in `~/.config/mpv/scripts` and fill it out with the values below

### Config

```toml
enable_on_start = true
server_url = "wss://server.syncwatch.space"
name = "<FILL THIS OUT>"
room_name = "<FILL THIS OUT>"
```

## Usage

Choose a name and have your friends connect to the same room and the plugin will automatically sync your playback every time someone pauses/plays.
