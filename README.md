# Rice Studio

**Pick a desktop theme, make it yours, add your own wallpapers, and apply it to Windows in one click.**

Rice Studio is a free, open-source Windows app that "rices" a desktop: it replaces the stock look with a complete,
coherent theme. One apply sets up a custom top bar with a dashboard, a desktop clock widget, a scrolling tiling
window manager, a themed launcher, Start menu and Explorer styling, window animations, terminal colours and the
wallpaper, and it can all be undone in one step.

> **Status: work in progress, private development repository.** The app runs and applies themes end to end on the author's machine. Installer
> packaging, signed releases and the in-app Claude assistant are still in progress. Expect breaking changes.

---

## Contents

- [Features](#features)
- [Themes](#themes)
- [How it works](#how-it-works)
- [Requirements](#requirements)
- [Building from source](#building-from-source)
- [Command-line interface](#command-line-interface)
- [Project layout](#project-layout)
- [Safety](#safety)
- [Development notes](#development-notes)
- [Licence](#licence)

---

## Features

- **Twelve hand-built themes.** Each one is a separate design rather than a recolour: its own bar layout,
  dashboard, power menu, wallpaper picker, shortcut sheet, desktop clock, typography, motion and workspace names.
- **Customise before you apply.** Edit the palette, fonts, bar height, motion and workspace names. A live native
  preview shows the bar, clock and wallpaper as they will look.
- **Bring your own wallpapers.** Drop in your own images. The theme lays them out for your screen and applies its
  own treatment (grain, halftone, oil, dither, and others) so any picture fits the look. Rice Studio ships no
  copyrighted art.
- **A 3D gallery.** The app's stage is a wgpu scene behind a Slint UI: theme cards are glass slabs on a helix
  around a rotating tesseract, with the camera travelling between pages.
- **Full desktop, not only a bar.** One apply configures:
  - [Zebar](https://github.com/glzr-io/zebar): bar, dashboard and desk widgets;
  - LeopardWM scrolling tiling: gaps, focus border, workspaces;
  - [Flow Launcher](https://github.com/Flow-Launcher/Flow.Launcher): search;
  - [Windhawk](https://github.com/ramensoftware/windhawk) mods: Start menu, File Explorer, window animations;
  - Windows Terminal colour schemes.
- **Undo everything.** Every apply is saved as a history step. Undo walks back one apply at a time, and the state
  from before the first apply is kept so stock Windows can always be restored.
- **Built to stay out of the way.** A small supervisor keeps the tools running, turns on a game mode for
  anti-cheat games, and holds a global panic hotkey.

## Themes

| Theme | Idea |
|---|---|
| Vagabond | Musashi's Five Rings in sumi ink and washi paper |
| Fresco | Wet plaster, painted one day's patch at a time |
| Relic | A vanitas shelf lit by one candle that burns down with the day |
| Inferno | Black shapes cut out against the fire |
| Schematic | Blueprint meets the battle painting |
| Marquee | Red and white crime-film poster |
| Dossier | Beige case file, red annotations |
| Xerox | Photocopied poster, toner and halftone |
| Etch | Engraving, hatching and one point of light |
| Dither | 1-bit, Bayer dots and light beams |
| Ghost Signal | Teal halftone, HUD glass, cyberpunk |
| Y2K Blue | Early-2000s blue pixel collage |

Every theme is defined in [`src/theme.rs`](src/theme.rs) (palette, fonts, bar style, motion, workspace names,
wallpaper treatment). Its widgets live in [`assets/themes/<id>/`](assets/themes).

## How it works

```
                 ┌──────────────────────────── Rice Studio (rice-studio.exe) ───────────────────────────┐
  Theme spec ──► │ theme.rs ──► zebar.rs      HTML/CSS/JS widget pack (bar, dashboard, desk)            │
  (presets +     │          ──► components.rs LeopardWM, Flow Launcher, Windhawk, terminal settings     │
   your edits)   │          ──► wallpaper.rs  your images + the theme's treatment, sized to the screen  │
                 │ apply.rs ──► writes everything, keeps a history step, restarts what needs it        │
                 │ system.rs──► supervisor: start order, health checks, game mode, panic hotkey        │
                 └───────────────────────────────────────────────────────────────────────────────────────┘
```

- **The app** is native Rust. The UI is [Slint](https://slint.dev); the 3D stage is drawn with wgpu into a texture
  that Slint shows behind the interface. Wallpapers, card faces and previews are painted on the CPU with
  [tiny-skia](https://github.com/RazrFalcon/tiny-skia), using real glyph outlines from the bundled fonts.
- **The widgets** run in Zebar, which renders web widgets. A shared engine
  ([`assets/engine/engine.js`](assets/engine/engine.js)) handles data (workspaces, media, system stats, weather,
  power, wallpapers, the shortcut sheet), and each theme supplies only its markup, styles and a small `voice.js`
  with its wording and visual hooks. Theme values are sanitised before they reach CSS or HTML.
- **Fonts** (about 40 open-licence families from Fontsource) are embedded in the executable and installed per
  user when a theme needs them.

## Requirements

- Windows 10 version 1809 or later, or Windows 11 (x64)
- [Rust](https://rustup.rs) stable, to build
- The bundled tools at runtime (Zebar, LeopardWM, Windhawk, Flow Launcher), placed in an `apps\` folder next to,
  or up to four levels above, `rice-studio.exe`. The installer script stages them; see
  [`installer/`](installer).

## Building from source

```powershell
git clone <this repository>
cd <repository folder>
cargo build --release
```

The executable is written to `target\release\rice-studio.exe`.

Run the tests:

```powershell
cargo test --release
```

Build the installer (requires [Inno Setup](https://jrsoftware.org/isinfo.php)):

```powershell
powershell -File installer\stage.ps1 -Apps <folder containing the bundled tools>
ISCC installer\rice-studio.iss
```

The installer is per-user, needs no administrator rights, and is written to `target\installer\`.

## Command-line interface

Starting `rice-studio.exe` with no arguments opens the app. The same work is available headless:

| Command | What it does |
|---|---|
| `--apply <theme-id>` | Apply a preset theme, e.g. `--apply relic` |
| `--undo` | Undo the last apply |
| `--wallpaper <n>` | Switch to the theme's wallpaper number *n* |
| `--supervise` | Run the background supervisor (started at sign-in) |
| `--autostart on\|off` | Start the supervisor with Windows, or stop doing so |
| `--panic` | Switch everything off and restore stock Windows |
| `--uninstall` | Restore stock Windows and remove installed fonts |
| `--render-test <dir>` | Render every theme's gallery card to PNG |
| `--pack-test <dir> <theme-id>` | Write a theme's widget pack to a folder, for testing in a browser |

Results of headless commands are also written to `data\cli-result.txt`.

## Project layout

```
assets/
  engine/        shared widget engine (data, dashboard, power, wallpaper picker, shortcut sheet)
  themes/<id>/   per-theme widgets: bar, dash and desk HTML/CSS, plus voice.js
  fonts/         embedded open-licence fonts (WOFF2)
  *.toml/.xaml   templates for LeopardWM and Flow Launcher
installer/       Inno Setup script, staging script and third-party notices
src/
  main.rs        app entry, UI wiring, background jobs, CLI
  theme.rs       the preset themes and input sanitising
  zebar.rs       theme to Zebar widget pack
  components.rs  theme to LeopardWM, Flow, Windhawk and terminal settings
  wallpaper.rs   wallpaper composition and treatments
  art.rs         each theme's own art (local pictures or code-drawn pieces)
  backgrounds.rs procedural backgrounds
  paint.rs       2D painter over tiny-skia
  preview.rs     native previews of the bar, clock and cards
  gpu.rs         wgpu stage: fog, glass cards, dust, tesseract, ink veil
  scene.rs       stage motion: helix, camera, springs
  fonts.rs       font library and per-user install
  apply.rs       apply, history, undo, stock snapshot
  system.rs      supervisor, health checks, game mode, panic hotkey
  shaders/       WGSL shaders
ui/app.slint     the interface
DESIGN.md        design notes for the stage and motion
```

## Safety

Theming a desktop touches a lot of running software, so Rice Studio is deliberately conservative:

- **Per-user only.** No administrator rights, no system files changed. The taskbar is auto-hidden, never removed.
- **Always reversible.** A snapshot before every apply, step-by-step undo, and a full restore to stock.
- **Panic hotkey.** `Ctrl+Alt+Shift+Q` switches everything off and gives Windows back, at any time.
- **Safe start.** If two sign-ins in a row fail to come up healthy, the next one starts plain, with a notice.
- **Anti-cheat aware.** When a game with kernel anti-cheat starts (Riot Vanguard, Easy Anti-Cheat, BattlEye and FACEIT),
  the tiler and Windhawk are switched off automatically and come back when the game exits. The bar stays.
- **Light on resources.** The 3D stage throttles from 60 fps down to 4 fps when idle or covered, and stops when
  minimised. Desktop widgets keep their ambient motion slow, and stop it under game mode or when Windows has "Show animations" off.

## Development notes

- **Widget design.** Theme widgets follow the [Hallmark](https://github.com/nutlope/hallmark) design rules. Each
  theme stylesheet opens with its design stamp, and is checked with Hallmark's `sloplint` until no failures remain.
- **Testing widgets.** `--pack-test` writes a theme's pack to a folder. Serve that folder over HTTP and open it in a
  browser to work on a bar or dashboard without touching the live desktop.
- **Wallpapers and art** a user adds are stored in `data\art\<theme-id>\` next to the executable. The installer never
  ships that folder.

## Theme art (development copy)

This repository carries the working art for the four themes that have their own pictures, so collaborators see the
themes exactly as designed. **It is reference material from the web, not licensed for redistribution, and must be
removed before the repository is made public or the app is released.**

```
art/
  fresco/   inferno/   relic/   vagabond/
    walls/  desktop wallpapers, in picker order (01-…, 02-…)
    ws/     one picture per workspace, shown on the dashboard (1-… to 5-…)
  sources/  the original portrait pictures for Fresco, Inferno and Relic
tools/
  compose.py  turns art/sources into the 16:9 walls and ws pictures, composed the way each theme hangs a picture
```

To use the art, copy each theme folder next to the built executable:

```powershell
cargo build --release
New-Item -ItemType Directory -Force target\release\data\art | Out-Null
Copy-Item art\fresco, art\inferno, art\relic, art\vagabond target\release\data\art -Recurse -Force
```

To recompose the Fresco, Inferno and Relic pictures from the sources (needs Python with Pillow and NumPy):

```powershell
python tools\compose.py art\sources art\composed
```

## Licence

Rice Studio is free software, licensed under the [GNU General Public License v3.0 or later](LICENSE).

It bundles third-party programs unchanged, each under its own licence (Zebar, LeopardWM and Windhawk under GPL-3.0,
Flow Launcher under MIT), and fonts under the SIL Open Font License 1.1 or Apache 2.0. See
[`installer/NOTICES.txt`](installer/NOTICES.txt) for details.
