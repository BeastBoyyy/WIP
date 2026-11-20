//! The rest of the desktop for a theme, beyond the bar and the wallpaper: LeopardWM (gaps, focus border, tab
//! strip, motion, workspace names), Flow Launcher (search box colours and type) and the Windhawk mods that style
//! the Start menu, Explorer and window animations. `build` turns a theme into these settings; each Apply stores
//! them in its history step and `write` puts them in place. `files` lists everything `write` can touch, so the
//! stock snapshot (apply.rs) can put each file back exactly as it was.
use crate::system::apps_dir;
use crate::theme::{Rgb, Theme};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

const LEOPARDWM: &str = include_str!("../assets/leopardwm.toml");
const FLOW: &str = include_str!("../assets/flow.xaml");
const FLOW_THEME: &str = "RiceStudio";
const MODS: [&str; 3] = ["windows-11-start-menu-styler", "windows-11-file-explorer-styler", "windows-animations"];

#[derive(Serialize, Deserialize, Default)]
pub struct Components {
    leopardwm: String,
    flow_xaml: String,
    flow_settings: Vec<(String, serde_json::Value)>,
    /// mod id → its [Settings] lines; an empty list leaves the mod at its own defaults
    windhawk: Vec<(String, Vec<(String, String)>)>,
    /// Windows Terminal fragment (colour scheme on the stock shells); steps saved before it have none
    #[serde(default)]
    terminal: String,
}

/// What text sits on: bg, unless the palette's ink is as dark as its bg (a light theme with dark mouldings), then surface.
fn ground(p: crate::theme::Palette) -> Rgb { if (p.bg.luma() - p.ink.luma()).abs() < 60. { p.surface } else { p.bg } }
/// The focused window's border: the accent, except Dossier's windows, which sit in manila like papers in a folder, and
/// Xerox's, which get a plain black toner line, Etch's, which sit in a hairline plate mark, and Dither's, one pixel of ink.
fn border(t: &Theme) -> Rgb { match t.id { "dossier" => t.palette.bg.mix(Rgb(0xC9, 0xA3, 0x5A), 0.6), "xerox" | "dither" | "ghost" => t.palette.ink, "etch" => t.palette.ink.mix(t.palette.bg, 0.3), "vagabond" => Rgb(0x54, 0x9E, 0xCE), "y2k" | "inferno" => t.palette.accent2, _ => t.palette.accent } }
fn hex(c: Rgb) -> String { format!("{:02X}{:02X}{:02X}", c.0, c.1, c.2) }
fn argb(a: u8, c: Rgb) -> String { format!("#{a:02X}{}", hex(c)) }
fn fill(mut s: String, vars: &[(&str, String)]) -> String {
    for (k, v) in vars { s = s.replace(&format!("{{{{{k}}}}}"), v); }
    s
}

pub fn build(t: &Theme) -> Components {
    let p = t.palette;
    let dark = p.bg.luma() < 128.;
    // (moves and scrolls, workspace switches and the overview) in ms
    let (move_ms, switch_ms) = match t.motion { "majestic" => (320, 420), "snappy" => (120, 160), "flicker" => (160, 200), "glitch" => (90, 120), _ => (200, 260) };
    let ws = t.ws.iter().map(|w| format!("\"{}\"", w.replace(['"', '\\'], ""))).collect::<Vec<_>>().join(", ");
    let leopardwm = fill(LEOPARDWM.to_string(), &[
        ("gap", "12".into()),
        ("border", hex(border(t))),
        ("tab_bg", hex(p.surface)),
        ("tab_active_bg", hex(p.surface.mix(p.ink, 0.12))),
        ("tab_active_text", hex(p.ink)),
        ("tab_inactive_text", hex(p.muted)),
        ("move_ms", move_ms.to_string()),
        ("switch_ms", switch_ms.to_string()),
        ("workspaces", ws),
    ]);
    let solid = |a: u8, c: Rgb| format!("<SolidColorBrush Color=\"{}\" />", argb(a, c));
    let g = ground(p);
    let stops = |v: &[(u8, Rgb, &str)]| v.iter().map(|(a, c, o)| format!("<GradientStop Color=\"{}\" Offset=\"{o}\" />", argb(*a, *c))).collect::<String>();
    let white = Rgb(255, 255, 255);
    let black = Rgb(0, 0, 0);
    // the launcher's frame: every theme its own card (radius, frame, ground, query size, item radius)
    let (radius, frame, frame_brush, card_brush, query_size, item_radius) = match t.id {
        // washi card with a lacquer-red rule along the top edge, nearly square corners like a cut sheet
        "vagabond" => ("3", "1 3 1 1",
            format!("<LinearGradientBrush MappingMode=\"Absolute\" StartPoint=\"0,0\" EndPoint=\"0,4\"><GradientStop Color=\"{}\" Offset=\"0\" /><GradientStop Color=\"{}\" Offset=\"0.74\" /><GradientStop Color=\"{}\" Offset=\"0.75\" /></LinearGradientBrush>",
                argb(0xFF, p.accent), argb(0xFF, p.accent), argb(0x22, p.ink)),
            format!("<LinearGradientBrush StartPoint=\"0,0\" EndPoint=\"0.3,1\"><GradientStop Color=\"{}\" Offset=\"0\" /><GradientStop Color=\"{}\" Offset=\"1\" /></LinearGradientBrush>",
                argb(0xF7, p.bg.mix(p.ink, 0.06)), argb(0xF7, p.bg)),
            "23", "2"),
        // a patch of fresh lime plaster, the wet azure of the day's giornata down its left edge
        "fresco" => ("2", "4 0 0 0", solid(0xFF, p.accent),
            format!("<LinearGradientBrush StartPoint=\"0,0\" EndPoint=\"0,1\">{}</LinearGradientBrush>", stops(&[(0xFA, p.surface, "0"),
                (0xFA, p.surface.mix(p.bg, 0.5), "0.6"), (0xFA, p.bg, "1")])),
            "22", "2"),
        // the dark of the vanitas shelf, one gilt line of candlelight along its top edge
        "relic" => ("2", "0 1 0 0", solid(0xFF, p.accent),
            format!("<LinearGradientBrush StartPoint=\"0,0\" EndPoint=\"0,1\">{}</LinearGradientBrush>", stops(&[(0xFA, p.surface, "0"),
                (0xFA, p.bg.mix(p.surface, 0.4), "0.45"), (0xFA, p.bg, "1")])),
            "22", "1"),
        // black at the top, the fire coming up from below, and a white-hot seam along the bottom edge
        "inferno" => ("3", "0 0 0 3",
            format!("<LinearGradientBrush StartPoint=\"0,0\" EndPoint=\"1,0\">{}</LinearGradientBrush>", stops(&[(0xFF, p.accent2, "0"),
                (0xFF, p.accent, "0.5"), (0xFF, p.accent2, "1")])),
            format!("<LinearGradientBrush StartPoint=\"0,0\" EndPoint=\"0,1\">{}</LinearGradientBrush>", stops(&[(0xF8, p.bg, "0"),
                (0xF8, p.bg.mix(p.surface, 0.14), "0.6"), (0xF8, p.bg.mix(p.surface, 0.42), "1")])),
            "22", "3"),
        // grid paper: slate ruled in faint blue squares, a hairline round the sheet and a yellow rule down its left edge
        "schematic" => ("0", "4 1 1 1",
            format!("<LinearGradientBrush MappingMode=\"Absolute\" StartPoint=\"0,0\" EndPoint=\"4,0\"><GradientStop Color=\"{}\" Offset=\"0\" /><GradientStop Color=\"{}\" Offset=\"0.99\" /><GradientStop Color=\"{}\" Offset=\"1\" /></LinearGradientBrush>",
                argb(0xFF, p.accent), argb(0xFF, p.accent), argb(0x8C, p.ink)),
            format!("<DrawingBrush TileMode=\"Tile\" Viewport=\"0,0,12,12\" ViewportUnits=\"Absolute\"><DrawingBrush.Drawing><DrawingGroup><GeometryDrawing Brush=\"{}\"><GeometryDrawing.Geometry><RectangleGeometry Rect=\"0,0,12,12\" /></GeometryDrawing.Geometry></GeometryDrawing><GeometryDrawing><GeometryDrawing.Pen><Pen Brush=\"{}\" Thickness=\"1\" /></GeometryDrawing.Pen><GeometryDrawing.Geometry><GeometryGroup><LineGeometry StartPoint=\"0,0.5\" EndPoint=\"12,0.5\" /><LineGeometry StartPoint=\"0.5,0\" EndPoint=\"0.5,12\" /></GeometryGroup></GeometryDrawing.Geometry></GeometryDrawing></DrawingGroup></DrawingBrush.Drawing></DrawingBrush>",
                argb(0xF6, p.bg.mix(p.surface, 0.6)), argb(0x1C, p.accent2)),
            "20", "0"),
        // a white poster card under a thick red rule, square-cut, the query set in the poster face
        "marquee" => ("0", "0 7 0 0", solid(0xFF, p.accent),
            format!("<LinearGradientBrush StartPoint=\"0,0\" EndPoint=\"0,1\">{}</LinearGradientBrush>", stops(&[(0xFC, p.surface, "0"),
                (0xFC, p.surface, "0.8"), (0xFC, p.bg, "1")])),
            "24", "0"),
        // an index card: a red margin rule down the left, a hairline round it, faint blue lines ruled across the stock
        "dossier" => ("1", "4 1 1 1",
            format!("<LinearGradientBrush MappingMode=\"Absolute\" StartPoint=\"0,0\" EndPoint=\"4,0\"><GradientStop Color=\"{}\" Offset=\"0\" /><GradientStop Color=\"{}\" Offset=\"0.99\" /><GradientStop Color=\"{}\" Offset=\"1\" /></LinearGradientBrush>",
                argb(0xFF, p.accent), argb(0xFF, p.accent), argb(0x40, p.ink)),
            format!("<DrawingBrush TileMode=\"Tile\" Viewport=\"0,0,24,24\" ViewportUnits=\"Absolute\"><DrawingBrush.Drawing><DrawingGroup><GeometryDrawing Brush=\"{}\"><GeometryDrawing.Geometry><RectangleGeometry Rect=\"0,0,24,24\" /></GeometryDrawing.Geometry></GeometryDrawing><GeometryDrawing><GeometryDrawing.Pen><Pen Brush=\"{}\" Thickness=\"1\" /></GeometryDrawing.Pen><GeometryDrawing.Geometry><LineGeometry StartPoint=\"0,23.5\" EndPoint=\"24,23.5\" /></GeometryDrawing.Geometry></GeometryDrawing></DrawingGroup></DrawingBrush.Drawing></DrawingBrush>",
                argb(0xFA, p.surface.mix(white, 0.55)), argb(0x30, Rgb(0x5C, 0x80, 0xA8))),
            "22", "0"),
        // a photocopied card: a hard black edge, toner speckled across the paper
        "xerox" => ("0", "2", solid(0xFF, p.ink),
            format!("<DrawingBrush TileMode=\"Tile\" Viewport=\"0,0,29,23\" ViewportUnits=\"Absolute\"><DrawingBrush.Drawing><DrawingGroup><GeometryDrawing Brush=\"{}\"><GeometryDrawing.Geometry><RectangleGeometry Rect=\"0,0,29,23\" /></GeometryDrawing.Geometry></GeometryDrawing><GeometryDrawing Brush=\"{}\"><GeometryDrawing.Geometry><GeometryGroup><EllipseGeometry Center=\"5,4\" RadiusX=\"0.8\" RadiusY=\"0.8\" /><EllipseGeometry Center=\"19,13\" RadiusX=\"0.6\" RadiusY=\"0.6\" /><EllipseGeometry Center=\"11,19\" RadiusX=\"1\" RadiusY=\"0.9\" /><EllipseGeometry Center=\"25,6\" RadiusX=\"0.5\" RadiusY=\"0.5\" /></GeometryGroup></GeometryDrawing.Geometry></GeometryDrawing></DrawingGroup></DrawingBrush.Drawing></DrawingBrush>",
                argb(0xFC, p.bg), argb(0x50, p.ink)),
            "22", "0"),
        // a black plate in a hairline plate mark, hatched faintly across with the burin
        "etch" => ("0", "1", solid(0xB0, p.ink),
            format!("<DrawingBrush TileMode=\"Tile\" Viewport=\"0,0,5,5\" ViewportUnits=\"Absolute\"><DrawingBrush.Drawing><DrawingGroup><GeometryDrawing Brush=\"{}\"><GeometryDrawing.Geometry><RectangleGeometry Rect=\"0,0,5,5\" /></GeometryDrawing.Geometry></GeometryDrawing><GeometryDrawing><GeometryDrawing.Pen><Pen Brush=\"{}\" Thickness=\"0.7\" /></GeometryDrawing.Pen><GeometryDrawing.Geometry><LineGeometry StartPoint=\"0,5\" EndPoint=\"5,0\" /></GeometryDrawing.Geometry></GeometryDrawing></DrawingGroup></DrawingBrush.Drawing></DrawingBrush>",
                argb(0xF8, p.bg), argb(0x12, p.ink)),
            "22", "0"),
        // a 1-bit dialog: the ground, a one-pixel frame of ink, square everywhere
        "dither" => ("0", "1", solid(0xFF, p.ink), solid(0xFF, p.bg), "22", "0"),
        // HUD glass: the dark surface nearly opaque, a teal hairline round it, the corners barely rounded
        "ghost" => ("2", "1", solid(0x80, p.ink), solid(0xF0, p.surface.mix(p.bg, 0.5)), "22", "2"),
        // a gel pill: blue deepening downward under a white sheen, a pale chrome rim, round everywhere
        "y2k" => ("22", "2", solid(0xFF, p.muted),
            format!("<LinearGradientBrush StartPoint=\"0,0\" EndPoint=\"0,1\"><GradientStop Color=\"{}\" Offset=\"0\" /><GradientStop Color=\"{}\" Offset=\"0.12\" /><GradientStop Color=\"{}\" Offset=\"0.55\" /><GradientStop Color=\"{}\" Offset=\"1\" /></LinearGradientBrush>",
                argb(0xF6, p.accent2.mix(Rgb(255, 255, 255), 0.25)), argb(0xF6, p.surface), argb(0xF8, p.surface.mix(p.bg, 0.55)), argb(0xFA, p.bg)),
            "22", "16"),
        _ => ("14", "1", solid(0x26, p.ink), solid(0xF2, g), "20", "8"),
    };
    let quiet = if matches!(t.id, "fresco" | "relic") { p.ink.mix(p.surface, 0.42) } else { p.muted };
    let flow_xaml = fill(FLOW.to_string(), &[
        ("dark", if g.luma() < 128. { "True" } else { "False" }.into()),
        ("ink", format!("#{}", hex(p.ink))),
        ("accent", format!("#{}", hex(p.accent))),
        ("muted", format!("#{}", hex(quiet))),
        ("muted_faint", argb(0x55, quiet)),
        ("radius", radius.into()), ("frame", frame.into()), ("frame_brush", frame_brush), ("card_brush", card_brush),
        ("query_size", query_size.into()), ("item_radius", item_radius.into()),
        ("line", format!("#{}", hex(p.surface.mix(p.ink, 0.1)))),
        ("highlight", format!("#{}", hex(match t.id { "vagabond" => p.accent2, "fresco" => p.accent.mix(black, 0.35), "relic" => p.accent, "inferno" => p.accent2, "schematic" | "marquee" | "dossier" | "etch" | "ghost" | "y2k" => p.accent, "xerox" => p.accent.mix(p.ink, 0.25), _ => p.accent.mix(p.ink, 0.35) }))),
        ("subtle", format!("#{}", hex(p.ink.mix(quiet, 0.5)))),
        ("selected", argb(0x40, p.accent)),
        ("thumb", format!("#{}", hex(p.surface.mix(p.ink, 0.2)))),
    ]);
    let font = |f: &str| serde_json::Value::String(f.to_string());
    let flow_settings = vec![
        ("Theme".into(), font(FLOW_THEME)),
        // a theme whose display face is its voice types the query in it
        ("QueryBoxFont".into(), font(match t.id { "vagabond" => &t.body, "fresco" | "schematic" | "marquee" | "xerox" | "etch" => &t.display, "dither" | "ghost" | "y2k" => &t.display, _ => &t.body })),
        ("ResultFont".into(), font(&t.body)),
        ("ResultSubFont".into(), font(&t.mono)),
        ("UseDropShadowEffect".into(), serde_json::Value::Bool(true)),
    ];
    // ponytail: the stylers' TintedGlass look is tuned for dark tints; light themes keep the stock Start menu and
    // Explorer until a light preset is designed
    let tint = format!("CommonBgBrush=<WindhawkBlur BlurAmount=\"18\" TintColor=\"{}\"/>", argb(0xD9, p.bg));
    let glass = |extra: &[(&str, &str)]| -> Vec<(String, String)> {
        if !dark { return vec![]; }
        let mut v = vec![("theme".into(), "TintedGlass".into()), ("styleConstants[0]".into(), tint.clone())];
        v.extend(extra.iter().map(|(k, v)| (k.to_string(), v.to_string())));
        v
    };
    let effect = match t.id { "vagabond" => Some("ink_splash"), "fresco" => Some("swell"), "relic" => Some("fade"), "inferno" => Some("scorch"), "schematic" => Some("stipple"), "marquee" => Some("splinter"), "dossier" => Some("swell"), "xerox" => Some("mirage"), "etch" => Some("stipple"), "dither" => Some("stipple"), "ghost" => Some("mirage"), "y2k" => Some("swell"), _ => None };
    // a theme may close its windows its own way too: Inferno's burn out like embers, Schematic's fold flat to a line,
    // Marquee's go like a shot pane of glass, Dossier's crumble away like old evidence, Xerox's jam like a bad copy,
    // Etch's dissolve into the grain of the plate, Dither's break up into dots, Ghost Signal's drop out like a lost feed
    let close = match t.id { "vagabond" => Some("perlin"), "fresco" => Some("perlin"), "relic" => Some("thanos"), "inferno" => Some("perlin"), "schematic" => Some("tv_off"), "marquee" => Some("shatter"), "dossier" => Some("thanos"), "xerox" => Some("glitch"), "etch" => Some("perlin"), "dither" => Some("pixel_melt"), "ghost" => Some("glitch"), "y2k" => Some("pixel_melt"), _ => None };
    let animations: Vec<(String, String)> = effect.map(|e| vec![
        ("minimize_restore.minimize_animation".into(), "1".into()), ("minimize_restore.restore_animation".into(), "1".into()),
        ("minimize_restore.launch_animation".into(), "1".into()), ("minimize_restore.random_effect".into(), "0".into()),
        ("minimize_restore.effect_style".into(), e.into()),
    ]).unwrap_or_default().into_iter().chain(close.into_iter().flat_map(|e| [("close.close_animation".into(), "1".into()),
        ("close.random_effect".into(), "0".into()), ("close.effect_style".into(), e.into())])).collect();
    Components {
        terminal: terminal(t),
        leopardwm,
        flow_xaml,
        flow_settings,
        windhawk: vec![
            (MODS[0].into(), glass(&[])),
            (MODS[1].into(), glass(&[("backgroundTranslucentEffect", "micaAlt")])),
            (MODS[2].into(), animations),
        ],
    }
}

/// The 16 terminal colours: hand-picked for a theme that has them, otherwise the usual hues pulled toward the palette.
fn ansi(t: &Theme) -> [Rgb; 16] {
    let h = |s: &str| Rgb::hex(s).unwrap();
    let p = t.palette;
    match t.id {
        // sumi black, lacquer red, moss, ochre, indigo, plum, verdigris, washi
        "vagabond" => ["#1b1d21", "#C8402E", "#7D8A5C", "#C8824B", "#56708A", "#8F5E72", "#5E8A84", "#CFC8B8",
                    "#6b6458", "#E0604C", "#A0AE7A", "#E0A66E", "#7E98B0", "#B07F94", "#84AEA8", "#F2EDE2"].map(h),
        // the fresco painter's pots: vine black, sinopia, verdaccio, giallorino, azzurro, lacca, smalt, lime white
        "fresco" => ["#2A303B", "#C2553E", "#7F9E78", "#E3C46A", "#5C8FD6", "#C0627E", "#6FB3BF", "#D9DEDF",
                     "#5B6472", "#E0745C", "#A2BF9A", "#F2DA8E", "#86AEEA", "#DC88A0", "#94CFD8", "#F4F6F5"].map(h),
        // a vanitas: bitumen, lacquer, olive, candle gilt, slate, dried rose, patina, bone
        "relic" => ["#2A2119", "#B4483A", "#8A8F5A", "#D9A848", "#6F8796", "#9C6E78", "#7E9E92", "#DCD2BE",
                    "#6B5B4A", "#D46A58", "#AEB27C", "#F0C66E", "#93AABA", "#BE909A", "#A2C0B4", "#F2EADA"].map(h),
        // embers: soot, flame, slag-green, brass, steel blue, temper purple, quench teal, ash-white
        "inferno" => ["#1C1410", "#D9502A", "#8A9A4E", "#F2A93B", "#5E7A96", "#9A5A78", "#6E9A92", "#E8D3B4",
                      "#5A4638", "#F0703E", "#A8B868", "#FFC86A", "#86A2BE", "#BE7E9C", "#92BEB4", "#FBEBD2"].map(h),
        // blueprint: slate, correction red, survey green, warning yellow, construction blue, violet, cyan, tracing white
        "schematic" => ["#1F2B35", "#D9574A", "#7FB07A", "#F2A900", "#6FA3D6", "#B48AC2", "#6CC0C4", "#D9D7CB",
                        "#4A5A66", "#F07A6C", "#9FD09A", "#FFC94A", "#96C0EE", "#D0A8DC", "#92DCDF", "#F4F2E8"].map(h),
        // a crime picture: projection black, poster red, a few washed-out colours of an old print, poster paper
        "marquee" => ["#1A1A1A", "#E3262E", "#8FA36B", "#E2B65A", "#6E8BB0", "#C4587A", "#7FA9A8", "#E4E0D6",
                      "#5E5A55", "#FF4A50", "#ADC287", "#F4CF7C", "#93AED0", "#E07C9C", "#9EC6C4", "#F4F1EA"].map(h),
        // a typed report on bond paper: ribbon black, correction red, carbon green, manila, carbon-copy blue, stamp-pad
        // violet, ledger teal, and a paper grey (the colours are inks, dark enough to read on the page)
        "dossier" => ["#1D1B18", "#A3161A", "#4E6B2E", "#8A5A12", "#1F4E8C", "#6E2E6A", "#1F6B6B", "#8A8478",
                      "#5C574F", "#C8282C", "#5F7F3A", "#A86E1A", "#2F64A8", "#8A3E86", "#2A8282", "#3A3732"].map(h),
        // toner on copier paper: the colours a cheap copy shop has, all of them dark enough to come out on the page
        "xerox" => ["#141414", "#B3261E", "#3E6B2A", "#8C6410", "#1E5A8C", "#7A2F74", "#2B7FA6", "#6E6A61",
                    "#4A4741", "#D2342B", "#4F8236", "#A87C1A", "#2A70AA", "#94408C", "#3B98C2", "#2A2826"].map(h),
        // a print, nearly all black and white: only the old inks faintly in it (sanguine, sap green, candle, slate,
        // dove, verdigris), each pale enough to sit in the monochrome, and the candle the one warm light
        "etch" => ["#1A1918", "#B07A6E", "#8E9A7A", "#D9C28E", "#7E8CA0", "#9C8494", "#84A0A0", "#CFCBC1",
                   "#55524D", "#CC9488", "#AAB696", "#F3DDA6", "#9CAABE", "#B8A0B0", "#A2BEBE", "#F2EFE8"].map(h),
        // a 1-bit screen has no colours: every slot is ink or ground, the dim ones a mid grey only where a program
        // insists on telling them apart (black, then white for every colour, grey for the dim ones)
        // the HUD: teal for the ordinary, amber for warnings and what matters, red for errors, all on near-black glass
        "ghost" => ["#0B1214", "#D8433A", "#5FB8A0", "#F0A23B", "#3E8FA8", "#B0607A", "#9CC9C0", "#C9DDD8",
                    "#2A3B3E", "#F06A5E", "#7FD8BE", "#FFC46B", "#62B4CC", "#D2849C", "#C2E8E0", "#EAF5F2"].map(h),
        // a 2001 desktop: navy ground, the colours bright and candy-like, blue and magenta the loudest
        "y2k" => ["#0B1170", "#FF4F8B", "#5CD67A", "#F2C23A", "#6C8CFF", "#E040A0", "#4FD6E8", "#DCE3FF",
                  "#3A47A8", "#FF7AAA", "#8AF0A2", "#FFDD6E", "#9CB2FF", "#FF77C8", "#8AEFFF", "#FFFFFF"].map(h),
        "dither" => ["#000000", "#FFFFFF", "#FFFFFF", "#FFFFFF", "#FFFFFF", "#FFFFFF", "#FFFFFF", "#FFFFFF",
                     "#808080", "#FFFFFF", "#FFFFFF", "#FFFFFF", "#FFFFFF", "#FFFFFF", "#FFFFFF", "#FFFFFF"].map(h),
        _ => {
            let bg = ground(p);
            let dark = bg.luma() < 128.;
            let base = ["#CC4B4B", "#6AA84F", "#D4A64B", "#4B7BCC", "#A45BC0", "#4BB3B3"].map(|c| {
                let c = h(c).mix(p.accent, 0.2);
                if dark { c.mix(p.ink, 0.12) } else { c.mix(Rgb(0, 0, 0), 0.25) }
            });
            let bright = base.map(|c| if dark { c.mix(Rgb(255, 255, 255), 0.25) } else { c.mix(Rgb(0, 0, 0), 0.2) });
            let mut v = [bg; 16];
            v[0] = p.surface; v[7] = p.ink.mix(bg, 0.15); v[8] = p.muted; v[15] = p.ink;
            v[1..7].copy_from_slice(&base); v[9..15].copy_from_slice(&bright);
            v
        }
    }
}
/// Profiles Windows Terminal makes for Windows PowerShell, PowerShell 7 and cmd: their GUIDs are fixed.
const SHELLS: [&str; 3] = ["{61c54bbd-c2c6-5271-96e7-009a87ff44bf}", "{574e775e-4f2a-5b96-ac1e-a2962a402336}", "{0caa0dad-35be-5f56-a8ff-afceeeaa6101}"];
fn terminal(t: &Theme) -> String {
    let p = t.palette;
    let c = |x: Rgb| format!("#{}", hex(x));
    let a = ansi(t);
    let names = ["black", "red", "green", "yellow", "blue", "purple", "cyan", "white"];
    // (ground, text): the theme's own, or what its palette gives
    let (bg, fg) = match t.id { "fresco" => (Rgb(0x16, 0x1B, 0x24), p.surface), "relic" => (p.bg, p.ink), "inferno" => (Rgb(0x14, 0x0C, 0x08), p.ink), "schematic" => (Rgb(0x13, 0x1C, 0x24), p.ink), "marquee" => (Rgb(0x0F, 0x0F, 0x0F), p.bg), "dossier" => (Rgb(0xEF, 0xEB, 0xE1), Rgb(0x22, 0x20, 0x1C)), "xerox" | "etch" | "dither" => (p.bg, p.ink), _ => (ground(p), p.ink) };
    let mut scheme = serde_json::json!({ "name": "Rice Studio", "background": c(bg), "foreground": c(fg),
        "cursorColor": c(p.accent), "selectionBackground": c(p.accent2.mix(bg, 0.45)) });
    for (i, n) in names.iter().enumerate() {
        scheme[*n] = c(a[i]).into();
        scheme[format!("bright{}{}", n[..1].to_uppercase(), &n[1..])] = c(a[i + 8]).into();
    }
    let profiles: Vec<serde_json::Value> = SHELLS.iter().map(|g| serde_json::json!({
        "updates": g, "colorScheme": "Rice Studio", "font": { "face": t.mono }, "padding": "12",
        "cursorShape": match t.id { "vagabond" => "filledBox", "fresco" => "vintage", "relic" => "underscore", "inferno" => "filledBox", "schematic" => "emptyBox", "marquee" => "doubleUnderscore", "dossier" => "underscore", "xerox" => "filledBox", _ => "bar" },
    })).collect();
    serde_json::to_string_pretty(&serde_json::json!({ "schemes": [scheme], "profiles": profiles })).unwrap()
}
pub fn terminal_fragment() -> PathBuf {
    PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or_default()).join("Microsoft").join("Windows Terminal").join("Fragments").join("RiceStudio").join("rice-studio.json")
}

fn leopardwm_config() -> PathBuf { PathBuf::from(std::env::var("APPDATA").unwrap_or_default()).join("leopardwm").join("config").join("config.toml") }
/// Flow's portable settings folder, when Flow is bundled
fn flow_userdata() -> Option<PathBuf> {
    let flow = apps_dir().join("flow");
    let mut apps: Vec<PathBuf> = fs::read_dir(&flow).ok()?.flatten().map(|d| d.path())
        .filter(|p| p.is_dir() && p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("app-"))).collect();
    apps.sort();
    Some(apps.pop()?.join("UserData"))
}
fn mod_ini(id: &str) -> PathBuf { apps_dir().join("windhawk").join("AppData").join("Engine").join("Mods").join(format!("{id}.ini")) }

/// Every file `write` may change.
pub fn files() -> Vec<PathBuf> {
    let mut v = vec![leopardwm_config()];
    if let Some(u) = flow_userdata() { v.push(u.join("Settings").join("Settings.json")); v.push(u.join("Themes").join(format!("{FLOW_THEME}.xaml"))); }
    v.extend(MODS.iter().map(|m| mod_ini(m)));
    v
}

/// Windhawk keeps its ini files in UTF-16 with a byte-order mark.
fn read_utf16(p: &PathBuf) -> Option<String> {
    let b = fs::read(p).ok()?;
    let w: Vec<u16> = b.get(2..)?.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
    Some(String::from_utf16_lossy(&w))
}
fn write_utf16(p: &PathBuf, s: &str) -> std::io::Result<()> {
    let mut b = vec![0xFF, 0xFE];
    for u in s.encode_utf16() { b.extend(u.to_le_bytes()); }
    fs::write(p, b)
}
/// keeps the mod's own [Mod] section, stamps the change time so the engine reloads it, replaces [Settings]
fn mod_text(old: &str, settings: &[(String, String)]) -> String {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let mut out = String::new();
    let mut in_mod = false;
    for line in old.lines() {
        let l = line.trim();
        if l.starts_with('[') { in_mod = l == "[Mod]"; if in_mod { out.push_str("[Mod]\r\n"); } continue; }
        if in_mod && !l.is_empty() && !l.starts_with("SettingsChangeTime=") { out.push_str(l); out.push_str("\r\n"); }
    }
    out.push_str(&format!("SettingsChangeTime={now}\r\n\r\n[Settings]\r\n"));
    for (k, v) in settings { out.push_str(&format!("{k}={v}\r\n")); }
    out
}

fn e<T: std::fmt::Display>(x: T) -> String { x.to_string() }

/// Puts the settings in place. Flow must be stopped first: it saves its own settings when it closes.
pub fn write(c: &Components) -> Result<(), String> {
    let lw = leopardwm_config();
    fs::create_dir_all(lw.parent().unwrap()).map_err(e)?;
    fs::write(&lw, &c.leopardwm).map_err(e)?;
    if let Some(u) = flow_userdata() {
        fs::create_dir_all(u.join("Themes")).map_err(e)?;
        fs::write(u.join("Themes").join(format!("{FLOW_THEME}.xaml")), &c.flow_xaml).map_err(e)?;
        let sp = u.join("Settings").join("Settings.json");
        fs::create_dir_all(sp.parent().unwrap()).map_err(e)?;
        let mut s: serde_json::Map<String, serde_json::Value> = fs::read_to_string(&sp).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
        for (k, v) in &c.flow_settings { s.insert(k.clone(), v.clone()); }
        fs::write(&sp, serde_json::to_string_pretty(&s).map_err(e)?).map_err(e)?;
    }
    // Rice Studio's own file: written for a theme, gone otherwise
    let wt = terminal_fragment();
    if c.terminal.is_empty() { let _ = fs::remove_dir_all(wt.parent().unwrap()); } else {
        fs::create_dir_all(wt.parent().unwrap()).map_err(e)?;
        fs::write(&wt, &c.terminal).map_err(e)?;
    }
    for (id, settings) in &c.windhawk {
        let p = mod_ini(id);
        // a mod that is not installed is skipped: there is no [Mod] section to keep
        let Some(old) = read_utf16(&p) else { continue };
        write_utf16(&p, &mod_text(&old, settings)).map_err(e)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_preset_fills_every_placeholder() {
        for t in crate::theme::presets().into_iter().flatten() {
            let c = build(&t);
            assert!(!c.leopardwm.contains("{{") && !c.flow_xaml.contains("{{"), "{}", t.id);
            assert!(c.leopardwm.contains(&format!("active_border_color = \"{}\"", hex(border(&t)))));
            assert!(c.leopardwm.parse::<toml::Table>().is_ok(), "{} leopardwm config is valid TOML", t.id);
            let wt: serde_json::Value = serde_json::from_str(&c.terminal).unwrap();
            assert_eq!(wt["schemes"][0].as_object().unwrap().len(), 21, "{}: name, 4 roles, 16 colours", t.id);
            assert!(c.flow_xaml.contains("<ResourceDictionary") && !c.flow_xaml.contains("{{"), "{}", t.id);
        }
    }
    #[test]
    fn mod_settings_keep_the_mod_section() {
        let old = "\r\n[Mod]\r\nLibraryFileName=x.dll\r\nDisabled=0\r\nSettingsChangeTime=1\r\n\r\n[Settings]\r\ntheme=Old\r\n";
        let s = mod_text(old, &[("theme".into(), "TintedGlass".into())]);
        assert!(s.contains("LibraryFileName=x.dll") && s.contains("Disabled=0") && s.contains("theme=TintedGlass"));
        assert!(!s.contains("theme=Old") && s.matches("SettingsChangeTime=").count() == 1);
        let p = std::env::temp_dir().join(format!("rs-ini-{}.ini", std::process::id()));
        write_utf16(&p, &s).unwrap();
        assert_eq!(read_utf16(&p).unwrap(), s);
        fs::remove_file(p).unwrap();
    }
}
