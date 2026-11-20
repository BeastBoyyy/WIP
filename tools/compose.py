# compose.py <src D:\themes> <out dir>: turns the user's portrait pictures into 16:9 wallpapers and workspace pictures
# for Fresco, Inferno and Relic, each composed the way its theme would hang a picture. Writes <out>\<id>\walls and \ws.
import sys, os
import numpy as np
from PIL import Image, ImageFilter, ImageEnhance

SRC, OUT = sys.argv[1], sys.argv[2]
W, H = 3072, 1728
rng = np.random.default_rng(7)

def load(theme, prefix):
    for f in sorted(os.listdir(os.path.join(SRC, theme))):
        if f.startswith(prefix):
            return Image.open(os.path.join(SRC, theme, f)).convert('RGB')
    raise SystemExit(f'missing {theme}/{prefix}')

def up(im, h):
    w = round(im.width * h / im.height)
    im = im.resize((w, h), Image.LANCZOS)
    return im.filter(ImageFilter.UnsharpMask(radius=2, percent=60, threshold=2))

def noise(w, h, scale):
    # smooth value noise: a small random grid blown up, so plaster mottles instead of specks
    g = rng.random((max(2, h // scale), max(2, w // scale))).astype(np.float32)
    return np.asarray(Image.fromarray((g * 255).astype(np.uint8)).resize((w, h), Image.BICUBIC), dtype=np.float32) / 255

def plaster(w, h):
    # lime plaster: a cool grey-white, mottled at three scales, a fine sand speck over it
    base = np.array([226, 229, 228], np.float32)
    m = noise(w, h, 260) * .5 + noise(w, h, 60) * .3 + noise(w, h, 9) * .2
    a = base[None, None, :] * (0.93 + 0.1 * m[..., None])
    a += (rng.random((h, w, 1)) - .5) * 7
    a[..., 2] += 2   # a touch cooler than paper
    return Image.fromarray(np.clip(a, 0, 255).astype(np.uint8))

def save(im, theme, kind, name, q=90):
    d = os.path.join(OUT, theme, kind); os.makedirs(d, exist_ok=True)
    im.save(os.path.join(d, name + '.jpg'), quality=q, optimize=True)

# ── Fresco ─────────────────────────────────────────────────────────────────────────────────────────────────
def ceiling(im):
    # a quadratura ceiling runs on: the painting, its mirror either side, so the vault reads as one
    h = H; s = up(im, h); m = s.transpose(Image.FLIP_LEFT_RIGHT)
    strip = Image.new('RGB', (s.width * 3, h)); strip.paste(m, (0, 0)); strip.paste(s, (s.width, 0)); strip.paste(m, (2 * s.width, 0))
    if strip.width < W:
        strip = up(strip, round(h * W / strip.width))
    x = (strip.width - W) // 2; y = (strip.height - H) // 2
    return strip.crop((x, y, x + W, y + H))

def fragment(im, cx=.58, hfrac=.8):
    # a fragment lifted from a wall and set into fresh lime plaster: a ragged break, the darker arriccio round it
    g = plaster(W, H)
    s = up(im, round(H * hfrac))
    if s.width > W * .72: s = up(im, round(H * hfrac * W * .72 / s.width))
    w, h = s.size; x0 = int(W * cx - w / 2); y0 = (H - h) // 2
    yy, xx = np.mgrid[0:h, 0:w].astype(np.float32)
    d = np.minimum(np.minimum(xx, w - 1 - xx), np.minimum(yy, h - 1 - yy))   # distance to the edge
    rag = noise(w, h, 40) * 26 + noise(w, h, 7) * 8   # how far the break bites in
    paint = np.clip((d - rag) / 2.0, 0, 1)
    rough = np.clip((d - rag + 14 + noise(w, h, 18) * 10) / 2.0, 0, 1) - paint   # the arriccio band
    a = np.asarray(g, np.float32)
    region = a[y0:y0 + h, x0:x0 + w]
    sv = np.asarray(s, np.float32)
    arr = np.array([168, 156, 140], np.float32) * (0.85 + 0.25 * noise(w, h, 5)[..., None])
    region = region * (1 - rough[..., None]) + arr * rough[..., None]
    region = region * (1 - paint[..., None]) + sv * paint[..., None]
    a[y0:y0 + h, x0:x0 + w] = region
    return Image.fromarray(np.clip(a, 0, 255).astype(np.uint8))

FRESCO = [('01-celestial-triumph', 'The Celestial', ceiling), ('02-gloria', '#renaissance', fragment),
          ('03-divine-providence', 'Cortona', ceiling), ('04-apotheosis', 'The Apotheosis', ceiling),
          ('05-the-creation', '408279', fragment), ('06-sant-ignazio', 'Andrea Pozzo', ceiling),
          ('07-grisaille', 'Rome Art', ceiling), ('08-il-gesu', 'Trionfo', ceiling),
          ('09-the-fragment', 'The worlds', fragment), ('10-lament', '830140', fragment),
          ('11-meeting-in-the-sky', 'Carta', lambda im: fragment(im, .58, .62))]
FRESCO_WS = [('1-azzurro', 'The Celestial'), ('2-verdaccio', 'The worlds'), ('3-sinopia', '830140'),
             ('4-giallorino', '#renaissance'), ('5-lacca', 'The Apotheosis')]

# ── Inferno ────────────────────────────────────────────────────────────────────────────────────────────────
def pyre(im, cx=.5):
    # the picture at full height; either side its own fire, blown up, blurred and banked down, so it burns edge to edge
    s = up(im, H)
    k = max(W / im.width, H / im.height) * 1.15
    bg = im.resize((round(im.width * k), round(im.height * k)), Image.BILINEAR)
    bg = bg.crop(((bg.width - W) // 2, (bg.height - H) // 2, (bg.width - W) // 2 + W, (bg.height - H) // 2 + H))
    bg = bg.filter(ImageFilter.GaussianBlur(110))
    bg = ImageEnhance.Brightness(bg).enhance(.42)
    a = np.asarray(bg, np.float32)
    # the sides go down into smoke: darker the further from the picture
    xx = np.abs(np.linspace(-1, 1, W))[None, :, None]
    a = a * (1 - .55 * np.clip((xx - .25) / .75, 0, 1) ** 1.3)
    w = s.width; x0 = int(W * cx - w / 2)
    t = np.linspace(0, 1, w)
    f = np.clip(np.minimum(t, 1 - t) / .3, 0, 1); f = f * f * (3 - 2 * f)
    sv = np.asarray(s, np.float32)
    a[:, x0:x0 + w] = a[:, x0:x0 + w] * (1 - f[None, :, None]) + sv * f[None, :, None]
    # a little heat from below: the floor of the picture is the brightest place
    yy = np.linspace(0, 1, H)[:, None, None]
    a = a * (0.9 + 0.12 * yy)
    return Image.fromarray(np.clip(a, 0, 255).astype(np.uint8))

INFERNO = [('01-eclipse', 'Angel', pyre), ('02-the-orchestra', '923730', pyre), ('03-the-stair', '418764', pyre),
           ('04-the-fall', '512143', pyre), ('05-the-sword', '635781', pyre), ('06-the-burning-temple', '706361', pyre),
           ('07-lucifer', '1009650', pyre), ('08-the-burning-head', '987625', pyre), ('09-three-halos', '94364', pyre),
           ('10-the-abyss', 'L Abisso', pyre), ('11-the-mountain', '\u2661', pyre)]
INFERNO_WS = [('1-acheron', '418764'), ('2-styx', '1009650'), ('3-phlegethon', '987625'), ('4-cocytus', '94364'), ('5-lethe', 'Angel')]

# ── Relic ──────────────────────────────────────────────────────────────────────────────────────────────────
BITUMEN = np.array([22, 17, 12], np.float32)
def shelf(im, crop_bottom=0.0):
    # tenebrism: the picture hangs at the right, full height, and its left side goes down into the dark
    if crop_bottom: im = im.crop((0, 0, im.width, round(im.height * (1 - crop_bottom))))
    s = up(im, H)
    if s.width > W * .62: s = s.crop(((s.width - int(W * .62)) // 2, 0, (s.width - int(W * .62)) // 2 + int(W * .62), H))
    a = np.tile(BITUMEN, (H, W, 1))
    w = s.width; x0 = min(W - w, int(W * .66 - w / 2))
    t = np.linspace(0, 1, w)
    f = np.clip(t / .4, 0, 1) ** 1.6 * np.clip((1 - t) / .22, 0, 1) ** 1.2
    yy = np.linspace(-1, 1, H)
    v = np.clip(1.15 - .35 * yy ** 2, 0, 1)   # the light falls off at the top and bottom too
    mask = f[None, :] * v[:, None]
    sv = np.asarray(s, np.float32)
    a[:, x0:x0 + w] = a[:, x0:x0 + w] * (1 - mask[..., None]) + sv * mask[..., None]
    a += (rng.random((H, W, 1)) - .5) * 3   # keeps the dark from banding
    return Image.fromarray(np.clip(a, 0, 255).astype(np.uint8))

RELIC = [('01-memento', '611011', shelf), ('02-the-key', '662521', shelf), ('03-the-crown', '939915', shelf),
         ('04-the-halo', '892275', shelf), ('05-saint-jerome', '914793', shelf), ('06-the-rosary', 'The relic', shelf),
         ('07-letters', '1126674', shelf), ('08-the-mask', '1059049', shelf), ('09-the-vault', 'Relic Vault', shelf),
         ('10-the-seer', '1034702', lambda im: shelf(im, .045))]
RELIC_WS = [('1-skull', 'The relic'), ('2-crown', '939915'), ('3-key', '662521'), ('4-book', '914793'), ('5-butterfly', '611011')]

which = sys.argv[3:] or ['fresco', 'inferno', 'relic']
for theme, walls, ws in [('fresco', FRESCO, FRESCO_WS), ('inferno', INFERNO, INFERNO_WS), ('relic', RELIC, RELIC_WS)]:
    if theme not in which: continue
    for name, pre, fn in walls:
        save(fn(load(theme, pre)), theme, 'walls', name); print(theme, name)
    for name, pre in ws:
        im = load(theme, pre)
        if pre == '1034702': im = im.crop((0, 0, im.width, round(im.height * .955)))
        save(im, theme, 'ws', name, 92)
