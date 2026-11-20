// Xerox — a punk zine in five issues: headings cut out of other magazines a letter at a time, everything else typed,
// a hand-written line where it matters, and one cyan.
import { $, set, store, esc } from './engine.js';
import { WS } from './gen.js';

const MON = ['JAN', 'FEB', 'MAR', 'APR', 'MAY', 'JUN', 'JUL', 'AUG', 'SEP', 'OCT', 'NOV', 'DEC'];
const DAYS = ['SUNDAY', 'MONDAY', 'TUESDAY', 'WEDNESDAY', 'THURSDAY', 'FRIDAY', 'SATURDAY'];
const DAY = ['SUN', 'MON', 'TUE', 'WED', 'THU', 'FRI', 'SAT'];
const MONTH = ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December'];
const pad = n => String(n).padStart(2, '0');
// what each issue is about
const TITLES = ['Static', 'Tape', 'Live', 'Concrete', 'Copy of a copy'];
const issue = i => String(WS[i] ?? `No. ${i + 1}`);
const SKY = { clear: 'sunny', cloudy: 'grey', light_rain: 'drizzle', heavy_rain: 'pissing down', snow: 'snow', thunder: 'thunder' };

// a word cut out of other magazines, letter by letter: each its own face, colours and tilt (the same text always
// comes out the same, so a heading does not shuffle every minute)
function ransom(text) {
  return [...text].map((ch, k) => {
    if (ch === ' ') return '<i class="sp"></i>';
    const h = ((k + 1) * 7919 + ch.charCodeAt(0) * 104729 + text.length * 31) % 1000;
    const rot = ((h * 13) % 100 / 100 - 0.5) * 12, dy = ((h * 29) % 100 / 100 - 0.5) * 3;
    return `<b class="cut f${h % 3} s${(h >> 3) % 4}" style="--r:${rot.toFixed(1)}deg;--y:${dy.toFixed(1)}px">${esc(ch)}</b>`;
  }).join('');
}
function cutInto(el, text) { if (el && el.dataset.t !== text) { el.dataset.t = text; el.innerHTML = ransom(text); } }

// a gauge as a ring of halftone dots: as many inked as the value, the rest faint (rebuilt only when the count moves)
function dots(svg, v, n) {
  if (!svg) return;
  const on = Math.round(Math.max(0, Math.min(100, v)) / 100 * n);
  if (svg.dataset.v === String(on)) return;
  svg.dataset.v = String(on);
  const vb = svg.viewBox.baseVal, c = vb.width / 2, r = c - c * 0.18, dr = c * (n > 16 ? 0.085 : 0.15);
  let s = '';
  for (let k = 0; k < n; k++) {
    const a = k / n * Math.PI * 2 - Math.PI / 2, x = c + Math.cos(a) * r, y = c + Math.sin(a) * r;
    s += `<circle class="${k < on ? 'on' : 'off'}" cx="${x.toFixed(2)}" cy="${y.toFixed(2)}" r="${(k < on ? dr : dr * 0.55).toFixed(2)}"/>`;
  }
  svg.innerHTML = s;
}

export default {
  ws: TITLES,
  day: DAY,
  dayLong: DAYS.map(s => s[0] + s.slice(1).toLowerCase()),
  clock: d => ({ hm: d.toTimeString().slice(0, 5), day: `${DAY[d.getDay()].toLowerCase()} ${d.getDate()} ${MON[d.getMonth()].toLowerCase()}` }),
  wx: SKY,
  // the scene, as the zine sees each program
  app: { windowsterminal: 'THE DESK', zen: 'THE WIRE', firefox: 'THE WIRE', chrome: 'THE WIRE', msedge: 'THE WIRE', brave: 'THE WIRE',
    opera: 'THE WIRE', code: 'PASTE-UP', cursor: 'PASTE-UP', explorer: 'THE BOX', files: 'THE BOX', spotify: 'THE STEREO',
    discord: 'THE SQUAT', telegram: 'THE SQUAT', claude: 'THE EDITOR', steam: 'THE ARCADE', notion: 'NOTES', obsidian: 'NOTES',
    settings: 'THE FUSEBOX', photoshop: 'PASTE-UP', blender: 'THE SHED', vlc: 'THE TELLY', obs64: 'ON AIR' },
  appWindow: 'NOW', appIdle: 'BLANK PAGE', idle: 'xerox',
  names: { windowsterminal: 'Terminal', msedge: 'Edge', code: 'VS Code', explorer: 'Files', claude: 'Claude', zen: 'Zen',
    chrome: 'Chrome', firefox: 'Firefox', brave: 'Brave', spotify: 'Spotify', discord: 'Discord', steam: 'Steam', obs64: 'OBS' },
  nowm: ['OUT OF TONER', 'the copier is off'],
  t: { cpu: 'Processor', ram: 'Memory', battery: 'Battery' },

  dash: {
    size: { dash: [900, 540], power: [440, 320], wall: [930, 250], keys: [900, 440] },
    closeMs: 160,
    date: d => { const i = +store('rs.ws') || 0; return [d.toTimeString().slice(0, 5), `${DAYS[d.getDay()]} ${d.getDate()} ${MONTH[d.getMonth()].toUpperCase()}`, `${issue(i).toUpperCase()} · ${TITLES[i % 5].toUpperCase()}`]; },
    month: (y, m) => `${MONTH[m].toLowerCase()} ${y}`, wd: ['s', 'm', 't', 'w', 't', 'f', 's'],
    rings: [['cpu', 'cpu', 'thinking'], ['ram', 'memory', 'remembering'], ['disk', 'disk', 'hoarded'], ['bat', 'battery', 'left in it']],
    idleMedia: ['NOTHING ON', 'put a record on'],
    mottos: ['Do it yourself.', 'Copy it. Pass it on.', 'Nobody asked. Print it anyway.', 'Loud is a colour.', 'Make it, then make it worse.',
      'Three chords and a photocopier.', 'If it jams, kick it.', 'Cut, paste, repeat.'],
    power: { lock: ['LOCKED OUT', 'Lock'], sleep: ['LIGHTS OUT', 'Sleep'], restart: ['REPRINT', 'Restart'], off: ['PULL THE PLUG', 'Shut down'],
      idle: ['LAST PAGE', 'How does this issue end?', 'Esc to keep reading'], confirm: 'slap it again to confirm', twice: 'slap it twice' },
    walls: [['No. 1', 'Static'], ['No. 2', 'Tape'], ['No. 3', 'Live'], ['No. 4', 'Concrete'], ['No. 5', 'Copy of a copy'], ['ZINE', 'Yours']],
    wallOn: 'IN PRINT', wallBusy: 'COPYING …', wallFail: 'paper jam',
    keys: {
      windows: ['p.1', 'Windows', 'Ctrl + Alt +'], workspaces: ['p.2', 'Issues', 'Ctrl + Alt +'], escape: ['p.3', 'When it jams'],
      bar: ['p.4', 'The bar', 'mouse'],
      barRows: [['folder', 'overview'], ['issues', 'go · wheel · right: send'], ['find · files · shell · keys', 'Flow · Explorer · Terminal · this'],
        ['now', 'wheel focus · 2× maximize'], ['', 'right: float · middle: close'], ['‹ tray', 'tray icons drawer'],
        ['stereo', 'play · right: next · mid: prev'], ['dots', 'Task Manager'], ['battery', 'battery settings'],
        ['volume', 'wheel · click mute · right: sound'], ['clock', 'dashboard · right: notifications'], ['sticker', 'lock · sleep · restart · off']],
    },
  },

  // the desk widget is a flyer stapled to the wall: the day cut out, the time on a cyan scrap
  desk(d) {
    cutInto($('dday'), `${DAY[d.getDay()]} ${d.getDate()} ${MON[d.getMonth()]}`);
    const t = $('dtime');
    if (t) { set(t, 'textContent', d.toTimeString().slice(0, 5)); t.classList.remove('jit'); void t.offsetWidth; t.classList.add('jit'); }
    if (!$('dno')?.textContent) this.fx.deskWs(+store('rs.ws') || 0);
  },

  fx: {
    // an issue: its number stamped big, "No." small; the open one cut in on cyan and run through the copier (bar.css)
    wsButton: n => { const m = String(n).match(/^(no\.?\s*)(.+)$/i); return m ? `<span class="no">No.</span><b>${esc(m[2])}</b>` : `<b>${esc(n)}</b>`; },
    ring: (svg, v) => dots(svg, v, 10),
    dashRingHTML: (id, k, en) => `<div class="ring2" id="g-${id}"><div class="dot"><svg viewBox="0 0 64 64" id="r-${id}"></svg><em id="v-${id}">–</em></div>` +
      `<div class="rt"><b>${k}</b><span id="s-${id}">${en}</span></div></div>`,
    dashRing(id, v, reset) { const s = $(`r-${id}`); if (reset && s) s.dataset.v = ''; dots(s, reset ? 0 : v, 24); },
    // the page goes through the copier: the lamp passes under it, the staples go in, the title is cut out fresh
    dashOpen(i) {
      cutInto($('artCut'), TITLES[i % 5].toUpperCase());
      const b = document.body; b.classList.remove('copy'); void b.offsetWidth; b.classList.add('copy');
    },
    dashReady() { cutInto($('pcut'), 'THE END?'); cutInto($('wcut'), 'BACK ISSUES'); cutInto($('kcut'), 'HOW TO'); },
    deskWs(i) { set($('dno'), 'textContent', `${issue(i)} — ${TITLES[i % 5].toLowerCase()}`); },
    deskWx(k, c) { set($('dwx'), 'textContent', `weather: ${SKY[k] ?? SKY.clear}, ${c}°`); },
  },
};
