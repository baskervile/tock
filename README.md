# Tock for macOS

A terminal clock, stopwatch and countdown timer with steady-width digits, RGB gradients and a built-in alert. Tuned for Apple silicon macOS and Ghostty; no custom sound files or Nerd Font are required.

## Install

```sh
brew install baskervile/tools/tock
```

If your shell still finds an older Cargo binary, check `type -a tock`. Put Homebrew's bin directory first in PATH, or update your Cargo installation from this repository.

## Commands

```sh
tock                         # clock, warm gradient
tock -s                      # stopwatch; Space starts it
tock -t 5m                   # five-minute countdown, starts immediately
tock -t 1h30m                # one hour thirty minutes
tock --mode countdown        # open countdown presets
tock --color viridis
tock --color '#89b4fa,#cba6f7'
tock --color none --fg cyan   # solid color
tock --silent                # no audio
tock --font small            # smaller ASCII digits
tock --hidden-help           # hide the footer
tock --config-dir            # print the active settings directory
tock --help
```

Durations use `h`, `m` and `s`; plain numbers and zero-length timers are rejected. `-s` starts paused. `-t` starts immediately. Close Tock to stop a timer: it is not a background alarm service, and sleeping macOS may delay an alert.

## Keys

| Key | Action |
| --- | --- |
| Tab | Clock → stopwatch → countdown |
| Space | Start/pause timer; stop an alert |
| r | Reset timer; stopwatch records the elapsed interval |
| p | Show/hide the panel |
| q, Esc, Ctrl+C | Quit outside text editing |
| Arrow keys | Move through panel rows/columns |
| g / G | First / last row |
| a / e / d | Add / edit / delete a panel item |
| Enter | Apply a countdown preset; confirm a field while editing |
| Esc while editing | Cancel editing |

To create a countdown preset: Tab to countdown, press p, then a. Enter a name and a duration **in seconds**, using Enter to advance fields. Afterwards select it, press Enter to load it, and Space to start. In alarm editing, Space toggles enabled and cycles repeat options in those columns.

Switching modes resets the current stopwatch and pauses the countdown. Alarms are checked while clock mode is active. Keep Tock in clock mode when using its alarms.

## Gradients

The default is `warm`. An explicit `--fg` selects a solid color unless a gradient is also supplied. Disable a configured gradient with `--color none`.

Presets: `rainbow`, `sinebow`, `viridis`, `magma`, `plasma`, `inferno`, `turbo`, `spectral`, `blues`, `greens`, `reds`, `oranges`, `purples`, `warm`, `cool`.

Custom gradients take two or more comma-separated HTML colors. Invalid names produce an error before entering full-screen mode.

## Settings and sound

Run `tock --config-dir` to find the active folder. New installations use `~/.config/tock` (or `$XDG_CONFIG_HOME/tock`). Existing macOS settings in `~/Library/Application Support/tock` remain supported when the new folder is absent. Settings, sound lookup and data storage all use that same directory.

Create `config.toml` there:

```toml
font = "standard"
bg = "reset"
color = "warm"
subtitle_fg = "cyan"
panel_ratio = 50
panel_border_style = "rounded"
silent = false
hidden_help = false
```

CLI options override settings. `data.json` stores alarms, countdown presets and stopwatch history. Back up that folder with your projects.

An alert is bundled into the binary and plays without setup. Audio opens only when an alert fires. To choose your own sound:

```sh
tock --alarm-sound "$HOME/Music/alarm.wav"
tock -t 5m --countdown-sound "$HOME/Music/timer.mp3"
```

You can also put WAV, MP3, OGG or FLAC files in the active folder's `sounds/` directory and refer to their name. Config keys are `alarm_sound` and `countdown_sound`. Omit these keys to use the bundled alert. Missing explicitly configured files produce an actionable error instead of startup warnings.

Fonts: built-in `standard`, `small`, `big`, `slant`, or an explicit `.flf` path. Use a full path for custom fonts.

## Ghostty

Use Ghostty's normal environment and leave `TERM` as `xterm-ghostty`; no color workaround is required. Tock outputs RGB colors directly. If your shell sets `NO_COLOR`, colors are intentionally disabled; try `env -u NO_COLOR tock --color warm`. Its reset background respects your terminal theme, mouse selection is not captured, and small windows automatically fall back to compact digits. Increase the window width if you want large digits. [Ghostty terminfo documentation](https://ghostty.org/docs/help/terminfo).

## Build

```sh
xcode-select --install
brew install rust
git clone https://github.com/baskervile/tock.git
cd tock
cargo test --locked
cargo install --path . --locked
```

## Provenance

Fork of [akirco/tock](https://github.com/akirco/tock). Custom digit rendering recovered from the owner's July development history; see [RECOVERY.md](RECOVERY.md). The bundled alert is an original synthesized WAV. MIT license; upstream copyright is retained.
