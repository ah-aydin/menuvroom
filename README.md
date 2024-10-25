# MenuVroom

Bare bones app launcher. I've made it as a replacement for dmenu on i3, but it can work in other desktop environments as well.

Build steps:
1. Clone repo
2. Run `cargo build --release`
3. Create keybinding
```
bindsym $mod+m exec /path/to_project/target/release/menuvroom
```

## Config

Example config (place it in `~/.config/menuvroom`). All values are optional.
```json
{
  "extra_directories": [
    "/var/lib/flatpak/exports/bin"
  ],
  "ignored_directories": [
    "/usr/local/games"
  ],
  // Will automatically create a file called `executables.txt`
  "cache_dir": "~/.cache/menuvroom",

  "window_width": 1000,
  "window_height": 600,
  "window_pos_x": 30,
  "window_pos_y": 100,

  "font_size": 30,
  "line_height": 42,
  // Values must be between 0 and 255
  "font_color": {
    "r": 0, "g": 255, "b": 0
  },
  // Values must be between 0 and 255
  "font_color_highlighted": {
    "r": 255, "g": 255, "b": 255
  },
  // Values must be between 0 and 1
  "bg_color": {
    "r": 0.05, "g": 0.05, "b": 0.05, "a": 0.9
  }
}
```
