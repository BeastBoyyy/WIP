//! Theme → the Zebar widget pack (HTML, CSS, JS, fonts). Zebar renders web widgets, so the bar it runs is
//! still HTML; this module only fills in the templates in assets/ with sanitized theme values.
use crate::apply::{Data, PackFile};
use crate::fonts;
use crate::theme::{sanitize, Theme};

// ponytail: the Vagabond install's tiler; the bundled app ships its own copy (Phase 2)
fn lwm() -> String { crate::system::lwm().to_string_lossy().into() }
const CSS: &str = include_str!("../assets/bar.css");
const JS: &str = include_str!("../assets/bar.js");
const LIB: &str = include_str!("../assets/zebar.js");

fn esc(s: &str) -> String {
    s.chars().map(|c| match c { '&' | '<' | '>' | '"' | '\'' => format!("&#{};", c as u32), _ => c.to_string() }).collect()
}
fn svg_url(s: &str) -> String {
    let enc: String = s.bytes().map(|b| match b {
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'!' | b'~' | b'*' | b'\'' | b'(' | b')' => (b as char).to_string(),
        _ => format!("%{b:02X}"),
    }).collect();
    format!("url(\"data:image/svg+xml,{enc}\")")
}
fn torn() -> String {
    let mut s: u64 = 7;
    let mut r = || { s = s * 16807 % 2147483647; s as f64 / 2147483647.0 };
    let mut pts = vec!["0% 0%".to_string(), "100% 0%".to_string()];
    let mut x = 100.0;
    while x >= 0.0 { pts.push(format!("{x}% {}%", 84.0 + r() * 16.0)); x -= 2.5; }
    format!("polygon({})", pts.join(","))
}

pub fn bar_css(raw: &Theme) -> String {
    let t = sanitize(raw);
    let p = t.palette;
    let ease = match t.motion { "majestic" => "cubic-bezier(.3,.6,.1,1) .8s", "snappy" => "cubic-bezier(.2,.9,.1,1) .14s", "flicker" => "ease-out .3s",
        "glitch" => "steps(3) .18s", _ => "cubic-bezier(.2,.7,.2,1) .45s" };
    let noise = svg_url("<svg xmlns='http://www.w3.org/2000/svg' width='160' height='160'><filter id='n'><feTurbulence type='fractalNoise' baseFrequency='.9' numOctaves='2' stitchTiles='stitch'/><feColorMatrix values='0 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 .16 0'/></filter><rect width='100%' height='100%' filter='url(#n)'/></svg>");
    let zig = svg_url(&format!("<svg xmlns='http://www.w3.org/2000/svg' width='12' height='48' viewBox='0 0 12 48'><path d='M6 0 L2 8 L9 15 L3 24 L10 32 L4 40 L7 48' fill='none' stroke='{}' stroke-width='2'/></svg>", p.accent.css()));
    let star = svg_url(&format!("<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 40 40'><path fill='{}' d='M20 0l3 13 11-8-7 12 13 3-13 3 7 12-11-8-3 13-3-13-11 8 7-12-13-3 13-3-7-12 11 8z'/></svg>", p.accent.css()));
    [("{{bg}}", p.bg.css()), ("{{surface}}", p.surface.css()), ("{{ink}}", p.ink.css()), ("{{accent2}}", p.accent2.css()), ("{{accent}}", p.accent.css()),
     ("{{muted}}", p.muted.css()), ("{{font_display}}", t.display.clone()), ("{{font_body}}", t.body.clone()), ("{{font_mono}}", t.mono.clone()),
     ("{{height}}", t.height.to_string()), ("{{ease}}", ease.into()), ("{{noise}}", noise), ("{{torn}}", torn()), ("{{zig}}", zig), ("{{star}}", star)]
        .iter().fold(CSS.to_string(), |s, (k, v)| s.replace(k, v))
}

pub fn bar_html(raw: &Theme) -> String {
    let t = sanitize(raw);
    let (h, m, date) = crate::theme::now_hm();
    let ws: String = t.ws.iter().enumerate().map(|(i, w)| {
        let c = if i == 0 { " on occ" } else if i == 2 { " occ" } else { "" };
        format!("<button class=\"w{c}\" data-i=\"{i}\">{}</button>", esc(w))
    }).collect();
    format!(r#"<div class="bar s-{} m-{}">
<div class="isl l"><span class="lbl">WORKSPACE</span><div class="ws" id="ws">{ws}</div></div>
<div class="isl c"><span class="lbl">DATE</span><span id="date">{}</span></div>
<div class="isl r"><span class="it"><span class="lbl">CPU</span><b id="cpu">12%</b></span><span class="it"><span class="lbl">MEM</span><b id="mem">41%</b></span><span class="it" id="batw"><span class="lbl">PWR</span><b id="bat">87%</b></span><span class="it clk"><span class="lbl">TIME</span><b id="clock">{}</b></span></div>
<i class="fx"></i></div>"#, t.style, t.motion, esc(&date), esc(&crate::theme::clock_text(&t, h, m)))
}

fn zpack(t: &Theme) -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "$schema": "https://github.com/glzr-io/zebar/raw/v3.0.0/resources/zpack-schema.json",
        "name": crate::apply::PACK, "version": "0.1.0", "description": format!("Rice Studio: {}", t.name), "tags": ["topbar"],
        "previewImages": [], "repositoryUrl": "",
        "widgets": [{
            "name": "bar", "htmlPath": "./bar.html", "zOrder": "normal", "shownInTaskbar": false, "focused": false, "resizable": false, "transparent": true,
            "includeFiles": ["*.html", "*.css", "*.js", "lib/*.js", "fonts/*.woff2"], "caching": { "defaultDuration": 0, "rules": [] },
            // the app writes privileges itself from a fixed list; packages and specs can never add any
            "privileges": { "shellCommands": [{ "program": lwm(), "argsRegex": "^(subscribe --events [a-z_,]+|workspace [1-9])$" }] },
            "presets": [{ "name": "default", "anchor": "top_left", "offsetX": "0px", "offsetY": "0px", "width": "100%", "height": format!("{}px", t.height + 4),
                "monitorSelection": { "type": "primary" }, "dockToEdge": { "enabled": true, "edge": "top", "windowMargin": "0px" } }]
        }]
    })).unwrap()
}

/// `wall` is the wallpaper made from your pictures: the picker offers it beside the theme's own art.
pub fn build_pack(raw: &Theme, wall: &tiny_skia::Pixmap) -> Vec<PackFile> {
    let t = sanitize(raw);
    let mut files = Vec::new();
    if let Some(l) = look(t.id) {
        let faces = font_files(&t, &mut files);
        let p = paths();
        let stub = |f: &str| format!("import V from './voice.js';\nimport {{ {f} }} from './engine.js';\n{f}(V);\n");
        for (path, text) in [
            ("zpack.json", zpack_full(&t, &p)), ("tokens.css", tokens(&t, &faces)), ("gen.js", gen_js(&t, &p)),
            ("engine.js", ENGINE.into()), ("voice.js", l.voice.into()), ("lib/zebar.js", LIB.into()),
            ("bar.html", l.bar_html.into()), ("bar.css", l.bar_css.into()), ("bar.js", stub("bar")),
            ("dash.html", l.dash_html.into()), ("dash.css", l.dash_css.into()), ("dash.js", stub("dash")),
            ("desk.html", l.desk_html.into()), ("desk.css", l.desk_css.into()), ("desk.js", stub("desk")),
        ] { files.push(PackFile { path: path.into(), data: Data::Text(text) }); }
        files.extend(art_files(&t, wall));
        return files;
    }
    let faces = font_files(&t, &mut files);
    let page = format!("<!doctype html><html><head><meta charset=\"utf-8\"><style>\n{faces}html,body{{margin:0;background:transparent;overflow:hidden}}\n{}\n</style></head><body>{}<script type=\"module\" src=\"./bar.js\"></script></body></html>",
        bar_css(&t), bar_html(&t));
    let js = JS.replace("{{lwm}}", &serde_json::to_string(&lwm()).unwrap()).replace("{{roman}}", if t.clock == "roman" { "true" } else { "false" });
    files.push(PackFile { path: "zpack.json".into(), data: Data::Text(zpack(&t)) });
    files.push(PackFile { path: "bar.html".into(), data: Data::Text(page) });
    files.push(PackFile { path: "bar.js".into(), data: Data::Text(js) });
    files.push(PackFile { path: "lib/zebar.js".into(), data: Data::Text(LIB.into()) });
    files
}

/// the theme's three families as woff2 files in the pack, and the @font-face rules that load them
fn font_files(t: &Theme, files: &mut Vec<PackFile>) -> String {
    let mut faces = String::new();
    let mut seen = Vec::new();
    for fam in [&t.display, &t.body, &t.mono] {
        if seen.contains(fam) { continue; }
        seen.push(fam.clone());
        // CJK families: Zebar falls back to the installed system font
        let (Some(slug), Some(w2)) = (fonts::slug_of(fam), fonts::woff2_of(fam)) else { continue };
        files.push(PackFile { path: format!("fonts/{slug}.woff2"), data: Data::Bytes(w2.to_vec()) });
        faces += &format!("@font-face{{font-family:'{fam}';src:url('./fonts/{slug}.woff2') format('woff2')}}\n");
    }
    faces
}

/// A theme with its own rice: its look and voice in assets/themes/<id>/, run by the shared engine.
struct Look { voice: &'static str, bar_html: &'static str, bar_css: &'static str, dash_html: &'static str, dash_css: &'static str, desk_html: &'static str, desk_css: &'static str }
macro_rules! look {
    ($id:literal) => { Look {
        voice: include_str!(concat!("../assets/themes/", $id, "/voice.js")),
        bar_html: include_str!(concat!("../assets/themes/", $id, "/bar.html")), bar_css: include_str!(concat!("../assets/themes/", $id, "/bar.css")),
        dash_html: include_str!(concat!("../assets/themes/", $id, "/dash.html")), dash_css: include_str!(concat!("../assets/themes/", $id, "/dash.css")),
        desk_html: include_str!(concat!("../assets/themes/", $id, "/desk.html")), desk_css: include_str!(concat!("../assets/themes/", $id, "/desk.css")),
    } };
}
fn look(id: &str) -> Option<Look> {
    Some(match id {
        "vagabond" => look!("vagabond"),
        "fresco" => look!("fresco"),
        "relic" => look!("relic"),
        "inferno" => look!("inferno"),
        "schematic" => look!("schematic"),
        "marquee" => look!("marquee"),
        "dossier" => look!("dossier"),
        "xerox" => look!("xerox"),
        "etch" => look!("etch"),
        "dither" => look!("dither"),
        "ghost" => look!("ghost"),
        "y2k" => look!("y2k"),
        _ => return None,
    })
}
const ENGINE: &str = include_str!("../assets/engine/engine.js");

/// every program a widget may run; the pack's privileges allow exactly these, with exact arguments
struct Paths { home: String, lwm: String, flow: String, explorer: String, taskmgr: String, terminal: String, rundll32: String, shutdown: String, powershell: String, studio: String }
fn paths() -> Paths {
    let env = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.into());
    let win = env("SystemRoot", r"C:\Windows");
    let sys = format!(r"{win}\System32");
    let s = |p: std::path::PathBuf| p.to_string_lossy().into_owned();
    let wt = std::path::Path::new(&env("LOCALAPPDATA", "")).join(r"Microsoft\WindowsApps\wt.exe");
    let powershell = format!(r"{sys}\WindowsPowerShell\v1.0\powershell.exe");
    Paths {
        home: env("USERPROFILE", r"C:\"), lwm: lwm(), flow: s(crate::system::apps_dir().join("flow").join("Flow.Launcher.exe")),
        explorer: format!(r"{win}\explorer.exe"), taskmgr: format!(r"{sys}\Taskmgr.exe"),
        terminal: if wt.is_file() { s(wt) } else { powershell.clone() },
        rundll32: format!(r"{sys}\rundll32.exe"), shutdown: format!(r"{sys}\shutdown.exe"), powershell,
        studio: std::env::current_exe().map(s).unwrap_or_default(),
    }
}
const SLEEP: &str = "-NoProfile -NonInteractive -Command Add-Type -AssemblyName System.Windows.Forms;[System.Windows.Forms.Application]::SetSuspendState('Suspend',$false,$false)";
fn exact(s: &str) -> String {
    let mut o = String::from("^");
    for ch in s.chars() { if "\\.+*?()|[]{}^$".contains(ch) { o.push('\\'); } o.push(ch); }
    o + "$"
}

fn rgb(c: crate::theme::Rgb) -> String { format!("{},{},{}", c.0, c.1, c.2) }
fn speed(motion: &str) -> &'static str { match motion { "majestic" => "1.5", "snappy" => ".65", "flicker" => ".85", "glitch" => ".55", _ => "1" } }
/// the palette, type and pace you set in Rice Studio, as CSS variables every theme file reads
fn tokens(t: &Theme, faces: &str) -> String {
    let p = t.palette;
    let mut s = format!("{faces}:root{{--bar-h:{}px;--speed:{};--display:'{}',serif;--body:'{}',serif;--mono:'{}',monospace;",
        t.height.saturating_sub(8), speed(t.motion), t.display, t.body, t.mono);
    for (k, c) in [("bg", p.bg), ("surface", p.surface), ("ink", p.ink), ("accent", p.accent), ("accent2", p.accent2), ("muted", p.muted)] {
        s += &format!("--{k}:{};--{k}-rgb:{};", c.css(), rgb(c));
    }
    s + "}\n"
}
fn gen_js(t: &Theme, p: &Paths) -> String {
    let j = |v: &str| serde_json::to_string(v).unwrap();
    format!("export const P = {{ home: {}, lwm: {}, flow: {}, explorer: {}, taskmgr: {}, terminal: {}, rundll32: {}, shutdown: {}, powershell: {}, studio: {} }};\nexport const WS = {};\nexport const BAR = {};\nexport const ART = {};\n",
        j(&p.home), j(&p.lwm), j(&p.flow), j(&p.explorer), j(&p.taskmgr), j(&p.terminal), j(&p.rundll32), j(&p.shutdown), j(&p.powershell), j(&p.studio),
        serde_json::to_string(&t.ws).unwrap(), t.height, art_js(t))
}
fn zpack_full(t: &Theme, p: &Paths) -> String {
    use serde_json::{json, Value};
    let cmd = |prog: &str, re: &str| json!({ "program": prog, "argsRegex": re });
    let widget = |name: &str, z: &str, focused: bool, art: bool, cmds: Vec<Value>, preset: Value| {
        let mut inc = vec!["*.html", "*.css", "*.js", "lib/*.js", "fonts/*.woff2"];
        if art { inc.push("art/*.jpg"); }
        json!({ "name": name, "htmlPath": format!("./{name}.html"), "zOrder": z, "shownInTaskbar": false, "focused": focused,
            "resizable": false, "transparent": true, "includeFiles": inc, "caching": { "defaultDuration": 0, "rules": [] },
            "privileges": { "shellCommands": cmds }, "presets": [preset] })
    };
    let preset = |anchor: &str, y: &str, w: &str, h: &str, dock: bool| json!({ "name": "default", "anchor": anchor, "offsetX": "0px", "offsetY": y,
        "width": w, "height": h, "monitorSelection": { "type": "primary" },
        "dockToEdge": { "enabled": dock, "edge": if dock { Some("top") } else { None }, "windowMargin": "0px" } });
    serde_json::to_string_pretty(&json!({
        "$schema": "https://github.com/glzr-io/zebar/raw/v3.0.0/resources/zpack-schema.json",
        "name": crate::apply::PACK, "version": "0.2.0", "description": format!("Rice Studio: {}", t.name), "tags": ["topbar"],
        "previewImages": [], "repositoryUrl": "",
        // the app writes privileges itself from this fixed list; packages and specs can never add any
        "widgets": [
            widget("bar", "normal", false, false, vec![
                cmd(&p.lwm, "^(subscribe --events [a-z_,]+|workspace [1-9]|move-to-workspace [1-9]|focus (left|right)|toggle-overview|toggle-floating|maximize-column|close-window|query all)$"),
                cmd(&p.explorer, "^(|ms-settings:(sound|batterysaver)|ms-actioncenter:|ms-availablenetworks:)$"),
                cmd(&p.taskmgr, "^$"), cmd(&p.terminal, "^$"), cmd(&p.flow, "^$"),
            ], preset("top_left", "0px", "100%", &format!("{}px", t.height + 6), true)),
            widget("dash", "top_most", true, true, vec![
                cmd(&p.rundll32, r"^user32\.dll,LockWorkStation$"), cmd(&p.shutdown, "^/[rs] /t 0$"), cmd(&p.powershell, &exact(SLEEP)),
                cmd(&p.studio, "^--wallpaper [0-9]{1,2}$"),
            ], preset("top_right", "-1200px", "800px", "500px", false)),
            // Schematic's desk is the wallpaper's own title block, live: it sits over the one drawn in the corner of each sheet
            widget("desk", "bottom_most", false, false, vec![], if t.id == "schematic" { preset("bottom_right", "0px", "25%", "100%", false) }
                else if t.id == "marquee" || t.id == "xerox" { preset("bottom_right", "0px", "560px", "100%", false) }
                else { preset("bottom_left", "0px", "560px", "100%", false) }),
        ]
    })).unwrap()
}
fn jpeg(px: &tiny_skia::Pixmap, q: u8) -> Vec<u8> {
    let rgb: Vec<u8> = px.data().chunks_exact(4).flat_map(|p| [p[0], p[1], p[2]]).collect();
    let mut out = Vec::new();
    let _ = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, q).encode(&rgb, px.width(), px.height(), image::ExtendedColorType::Rgb8);
    out
}
/// what the dashboard knows of the theme's own pictures: the wallpapers' names (from their file names, with any
/// leading number dropped) and how many workspace pictures there are
fn art_js(t: &Theme) -> String {
    let name = |p: &std::path::PathBuf| p.file_stem().map(|s| s.to_string_lossy().trim_start_matches(|c: char| c.is_ascii_digit() || c == '-' || c == '_' || c == ' ').replace(['-', '_'], " ")).unwrap_or_default();
    let walls: Vec<String> = crate::art::own(t.id, "walls").iter().take(30).map(name).collect();
    serde_json::json!({ "walls": walls, "ws": crate::art::own(t.id, "ws").len().min(9) }).to_string()
}
/// every wallpaper at screen size with a card of it (the drawn pieces, or your pictures), a card of the wallpaper made
/// from your pictures, and the workspace pictures for the dashboard
fn art_files(t: &Theme, wall: &tiny_skia::Pixmap) -> Vec<PackFile> {
    let (w, h) = crate::screen();
    let n = crate::art::count(t);
    let own = crate::art::own(t.id, "walls");
    let own = &own;
    let mut files: Vec<PackFile> = Vec::new();
    // four at a time: a 4K picture decoded is 33 MB
    for chunk in (0..n).collect::<Vec<_>>().chunks(4) {
        files.extend(std::thread::scope(|s| {
            let hs: Vec<_> = chunk.iter().map(|&i| s.spawn(move || {
                let (full, card) = match own.get(i).and_then(|p| crate::art::open(p)) {
                    Some(img) => (crate::art::fill(&img, w, h), crate::art::fill(&img, 480, 270)),
                    None => (crate::art::piece(t, i, w, h), crate::art::piece(t, i, 480, 270)),
                };
                [PackFile { path: format!("art/wall-{i}.jpg"), data: Data::Bytes(jpeg(&full, 92)) },
                 PackFile { path: format!("art/card-{i}.jpg"), data: Data::Bytes(jpeg(&card, 88)) }]
            })).collect();
            hs.into_iter().flat_map(|h| h.join().unwrap()).collect::<Vec<_>>()
        }));
    }
    let mut c = crate::paint::Cv::new(480, 270);
    c.draw_image(wall, 0., 0., 480., 270., true);
    files.push(PackFile { path: format!("art/card-{n}.jpg"), data: Data::Bytes(jpeg(&c.px, 88)) });
    for (i, p) in crate::art::own(t.id, "ws").iter().take(9).enumerate() {
        let Some(img) = crate::art::open(p) else { continue };
        let hh = img.height().min(1200);
        let ww = (img.width() as u64 * hh as u64 / img.height() as u64).max(1) as u32;
        let px = crate::paint::to_pixmap(&image::imageops::resize(&img, ww, hh, image::imageops::FilterType::CatmullRom));
        files.push(PackFile { path: format!("art/ws-{i}.jpg"), data: Data::Bytes(jpeg(&px, 90)) });
    }
    files
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_preset_compiles_without_leftover_placeholders() {
        let wall = tiny_skia::Pixmap::new(16, 9).unwrap();
        for t in crate::theme::presets().into_iter().flatten() {
            for f in super::build_pack(&t, &wall) {
                if let crate::apply::Data::Text(s) = &f.data { assert!(!s.contains("{{"), "{} {} has a placeholder", t.id, f.path); }
            }
        }
    }
    #[test]
    fn the_sleep_command_matches_its_own_privilege() {
        let re = super::exact(super::SLEEP);
        assert!(re.starts_with("^-NoProfile") && re.contains(r"\[System\.Windows\.Forms\.Application\]::SetSuspendState\('Suspend',\$false,\$false\)"));
    }
}
