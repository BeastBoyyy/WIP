//! The bundled font library (open licences, from Fontsource). WOFF files are embedded and unpacked to
//! plain sfnt at start-up, then registered with Slint and kept for the CPU painter.
use std::collections::HashMap;
use std::io::Read;
use std::sync::OnceLock;

macro_rules! fonts {
    ($( $fam:literal, $pkg:literal, $group:literal );* $(;)?) => {
        /// (family, package, group, woff, woff2)
        pub const LIBRARY: &[(&str, &str, &str, &[u8], &[u8])] = &[
            $( ($fam, $pkg, $group, include_bytes!(concat!("../assets/fonts/", $pkg, ".woff")), include_bytes!(concat!("../assets/fonts/", $pkg, ".woff2"))) ),*
        ];
    };
}
fonts! {
    "Cinzel", "cinzel", "Classical serif"; "Cormorant Garamond", "cormorant-garamond", "Classical serif";
    "Playfair Display", "playfair-display", "Classical serif"; "IM Fell English", "im-fell-english", "Classical serif";
    "IM Fell English SC", "im-fell-english-sc", "Classical serif"; "Fraunces", "fraunces", "Classical serif";
    "Libre Caslon Text", "libre-caslon-text", "Classical serif"; "Marcellus", "marcellus", "Classical serif";
    "Castoro", "castoro", "Classical serif"; "Castoro Italic", "castoro-italic", "Classical serif";
    "Grenze", "grenze", "Classical serif"; "UnifrakturCook", "unifrakturcook", "Blackletter";
    "Alegreya Sans", "alegreya-sans", "Humanist sans";
    "Red Hat Mono", "red-hat-mono", "Typewriter and mono"; "Chivo Mono", "chivo-mono", "Typewriter and mono"; "DM Mono", "dm-mono", "Typewriter and mono";
    "Anton", "anton", "Poster condensed"; "Bebas Neue", "bebas-neue", "Poster condensed";
    "League Gothic", "league-gothic", "Poster condensed"; "Oswald", "oswald", "Poster condensed";
    "Archivo Narrow", "archivo-narrow", "Grotesk"; "Space Grotesk", "space-grotesk", "Grotesk"; "Inter", "inter", "Grotesk";
    "Courier Prime", "courier-prime", "Typewriter and mono"; "Special Elite", "special-elite", "Typewriter and mono";
    "IBM Plex Mono", "ibm-plex-mono", "Typewriter and mono"; "Space Mono", "space-mono", "Typewriter and mono";
    "JetBrains Mono", "jetbrains-mono", "Typewriter and mono";
    "Alfa Slab One", "alfa-slab-one", "Distressed and slab"; "Rubik Dirt", "rubik-dirt", "Distressed and slab";
    "Chakra Petch", "chakra-petch", "Tech and HUD"; "Rajdhani", "rajdhani", "Tech and HUD"; "Share Tech Mono", "share-tech-mono", "Tech and HUD";
    "VT323", "vt323", "Pixel"; "Silkscreen", "silkscreen", "Pixel"; "Pixelify Sans", "pixelify-sans", "Pixel";
    "Yuji Syuku", "yuji-syuku", "Japanese"; "Shippori Mincho B1", "shippori-mincho-b1", "Japanese"; "Noto Sans JP", "noto-sans-jp", "Japanese";
}
const UI_WEIGHTS: &[&[u8]] = &[include_bytes!("../assets/fonts/inter-600.woff"), include_bytes!("../assets/fonts/space-grotesk-600.woff")];

/// WOFF 1.0 → sfnt (TrueType/OpenType). Tables are zlib-compressed one by one.
pub fn woff_to_sfnt(w: &[u8]) -> Option<Vec<u8>> {
    let u32_at = |o: usize| w.get(o..o + 4).map(|b| u32::from_be_bytes(b.try_into().unwrap()));
    let u16_at = |o: usize| w.get(o..o + 2).map(|b| u16::from_be_bytes(b.try_into().unwrap()));
    if u32_at(0)? != 0x774F_4646 { return None; }
    let flavor = u32_at(4)?;
    let n = u16_at(12)? as usize;
    let mut sel = 0u16;
    while (2usize << sel) <= n { sel += 1; }
    let range = (1u16 << sel) * 16;
    let mut out = Vec::new();
    out.extend(flavor.to_be_bytes());
    out.extend((n as u16).to_be_bytes());
    out.extend(range.to_be_bytes());
    out.extend(sel.to_be_bytes());
    out.extend(((n as u16) * 16 - range).to_be_bytes());
    let mut dir = Vec::new();
    let mut data = Vec::new();
    let mut offset = 12 + 16 * n;
    for i in 0..n {
        let e = 44 + i * 20;
        let (tag, off, clen, olen, sum) = (u32_at(e)?, u32_at(e + 4)? as usize, u32_at(e + 8)? as usize, u32_at(e + 12)? as usize, u32_at(e + 16)?);
        let raw = w.get(off..off + clen)?;
        let mut table = Vec::with_capacity(olen);
        if clen < olen { flate2::read::ZlibDecoder::new(raw).read_to_end(&mut table).ok()?; } else { table.extend_from_slice(raw); }
        dir.extend(tag.to_be_bytes());
        dir.extend(sum.to_be_bytes());
        dir.extend((offset as u32).to_be_bytes());
        dir.extend((olen as u32).to_be_bytes());
        let pad = (4 - olen % 4) % 4;
        data.extend(table);
        data.extend(std::iter::repeat_n(0u8, pad));
        offset += olen + pad;
    }
    out.extend(dir);
    out.extend(data);
    Some(out)
}

pub struct Fonts {
    /// family → sfnt bytes (leaked: they live for the whole run and Slint wants &'static)
    pub by_family: HashMap<&'static str, &'static [u8]>,
    /// a system CJK face for glyphs the bundled latin subsets lack (kanji workspace labels)
    pub cjk: Option<(&'static [u8], u32)>,
}

pub fn get() -> &'static Fonts {
    static F: OnceLock<Fonts> = OnceLock::new();
    F.get_or_init(|| {
        let mut by_family = HashMap::new();
        for (fam, _, _, woff, _) in LIBRARY {
            if let Some(s) = woff_to_sfnt(woff) { by_family.insert(*fam, &*Box::leak(s.into_boxed_slice())); }
        }
        // ponytail: Yu Gothic ships with every Windows 10/11; ship a CJK subset if a stripped install lacks it
        let cjk = ["C:\\Windows\\Fonts\\YuGothM.ttc", "C:\\Windows\\Fonts\\msgothic.ttc"].iter()
            .find_map(|p| std::fs::read(p).ok()).map(|b| (&*Box::leak(b.into_boxed_slice()), 0));
        Fonts { by_family, cjk }
    })
}

/// Make every bundled family (plus the UI's 600 weights) available to Slint text.
pub fn register_with_slint() {
    use slint::fontique_011::fontique::Blob;
    let f = get();
    let mut coll = slint::fontique_011::shared_collection();
    for bytes in f.by_family.values() { coll.register_fonts(Blob::new(std::sync::Arc::new(*bytes)), None); }
    for w in UI_WEIGHTS { if let Some(s) = woff_to_sfnt(w) { coll.register_fonts(Blob::new(std::sync::Arc::new(s)), None); } }
}

pub fn woff2_of(family: &str) -> Option<&'static [u8]> {
    LIBRARY.iter().find(|f| f.0 == family).map(|f| f.4)
}
pub fn slug_of(family: &str) -> Option<&'static str> {
    LIBRARY.iter().find(|f| f.0 == family).map(|f| f.1)
}

const FONTS_KEY: &str = r"HKCU\Software\Microsoft\Windows NT\CurrentVersion\Fonts";
const TAG: &str = "Rice Studio ";

fn reg(args: &[&str]) -> Option<String> {
    use std::os::windows::process::CommandExt;
    let o = std::process::Command::new("reg").args(args).creation_flags(0x0800_0000).output().ok()?;
    o.status.success().then(|| String::from_utf8_lossy(&o.stdout).into())
}
fn font_changed() {
    use windows_sys::Win32::UI::WindowsAndMessaging::{SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_FONTCHANGE};
    unsafe { SendMessageTimeoutW(HWND_BROADCAST, WM_FONTCHANGE, 0, 0, SMTO_ABORTIFHUNG, 1000, std::ptr::null_mut()) };
}
fn wide(s: &std::path::Path) -> Vec<u16> { use std::os::windows::ffi::OsStrExt; s.as_os_str().encode_wide().chain([0]).collect() }
/// .otf for CFF outlines, .ttf for TrueType ones
fn ext(sfnt: &[u8]) -> &'static str { if sfnt.starts_with(b"OTTO") { "otf" } else { "ttf" } }

/// Install bundled families for this user only (no admin): files under %LOCALAPPDATA%, one HKCU value each.
pub fn install(families: &[&str]) -> Result<(), String> {
    let dir = std::path::PathBuf::from(std::env::var("LOCALAPPDATA").map_err(|e| e.to_string())?).join(r"Microsoft\Windows\Fonts");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut changed = false;
    for fam in families {
        let (Some(bytes), Some(slug)) = (get().by_family.get(fam), slug_of(fam)) else { continue };
        let path = dir.join(format!("RiceStudio-{slug}.{}", ext(bytes)));
        if path.exists() { continue; }
        std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
        let name = format!("{TAG}{fam} (TrueType)");
        reg(&["add", FONTS_KEY, "/v", &name, "/t", "REG_SZ", "/d", &path.to_string_lossy(), "/f"]).ok_or("could not register a font")?;
        unsafe { windows_sys::Win32::Graphics::Gdi::AddFontResourceW(wide(&path).as_ptr()) };
        changed = true;
    }
    if changed { font_changed(); }
    Ok(())
}

/// Remove every font Rice Studio installed (uninstall).
pub fn uninstall_all() {
    let Some(list) = reg(&["query", FONTS_KEY]) else { return };
    for line in list.lines().map(str::trim).filter(|l| l.starts_with(TAG)) {
        let Some((name, path)) = line.split_once("    REG_SZ    ") else { continue };
        let p = std::path::Path::new(path.trim());
        unsafe { windows_sys::Win32::Graphics::Gdi::RemoveFontResourceW(wide(p).as_ptr()) };
        let _ = reg(&["delete", FONTS_KEY, "/v", name.trim(), "/f"]);
        let _ = std::fs::remove_file(p);
    }
    font_changed();
}

#[cfg(test)]
mod tests {
    #[test]
    fn font_files_get_the_right_extension() {
        assert_eq!(super::ext(b"OTTO...."), "otf");
        assert_eq!(super::ext(&[0, 1, 0, 0, 0]), "ttf");
    }
    #[test]
    fn every_bundled_woff_unpacks_to_a_parseable_font() {
        for (fam, _, _, woff, _) in super::LIBRARY {
            let s = super::woff_to_sfnt(woff).unwrap_or_else(|| panic!("{fam} did not unpack"));
            let face = ttf_parser::Face::parse(&s, 0).unwrap_or_else(|e| panic!("{fam}: {e}"));
            assert!(face.glyph_index('A').is_some(), "{fam} has no A");
        }
    }
}
