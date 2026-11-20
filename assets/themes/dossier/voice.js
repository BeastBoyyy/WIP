// Dossier — one case file in five tabs: typed labels with their index numbers, red marker for what matters, rubber
// stamps for what is final.
import { $, set, store, esc } from './engine.js';
import { WS } from './gen.js';

const MON = ['JAN', 'FEB', 'MAR', 'APR', 'MAY', 'JUN', 'JUL', 'AUG', 'SEP', 'OCT', 'NOV', 'DEC'];
const MONTH = ['JANUARY', 'FEBRUARY', 'MARCH', 'APRIL', 'MAY', 'JUNE', 'JULY', 'AUGUST', 'SEPTEMBER', 'OCTOBER', 'NOVEMBER', 'DECEMBER'];
const DAYS = ['SUNDAY', 'MONDAY', 'TUESDAY', 'WEDNESDAY', 'THURSDAY', 'FRIDAY', 'SATURDAY'];
const DAY = ['SUN', 'MON', 'TUE', 'WED', 'THU', 'FRI', 'SAT'];
const pad = n => String(n).padStart(2, '0');
// what is in each tab of the file
const NOTES = ['The door handle', 'Five faces', 'The night of the 13th', 'Typed and redacted', 'Filed away'];
const tab = i => String(WS[i] ?? `File ${i + 1}`).toUpperCase();
const exhibit = i => `EXHIBIT ${String.fromCharCode(65 + (i % 26))}`;
const SKY = { clear: 'CLEAR', cloudy: 'OVERCAST', light_rain: 'DRIZZLE', heavy_rain: 'HEAVY RAIN', snow: 'SNOW', thunder: 'STORM' };
// the mottos, typed; [bracketed] words come back blacked out
const MOTTOS = ['Everybody [lies]. The evidence [doesn\'t].', 'Follow the [money], then the [string].', 'Nothing is [closed] until it is [filed].',
  'The [witness] saw [everything].', 'Read it [twice]. Trust it [once].', 'Every [door] leaves a [print].', 'Somebody always [keeps] the [key].',
  'Coffee, [carbon paper], and the [truth].'];

// a ring in red marker: a dotted guide, and a loop drawn round as far as the value (pathLength 100, so the dash is the value)
function ring(svg, v, big) {
  if (!svg) return;
  const key = String(Math.round(v));
  if (svg.dataset.v === key) return;
  const vb = svg.viewBox.baseVal, c = vb.width / 2, r = c - (big ? 4 : 2);
  if (!svg.dataset.v) {
    // a marker loop is never quite round: a little wider than tall, tilted, and starting just past the top
    const pts = [];
    for (let s = 0; s <= 40; s++) {
      const a = -Math.PI / 2 + 0.2 + s / 40 * Math.PI * 2.08, k = 1 + Math.sin(s / 40 * 7) * 0.035;
      pts.push(`${(c + Math.cos(a) * r * 1.04 * k).toFixed(2)} ${(c + Math.sin(a) * r * 0.96 * k).toFixed(2)}`);
    }
    svg.innerHTML = `<circle class="base" cx="${c}" cy="${c}" r="${r}"/><path class="mk" pathLength="100" stroke-dasharray="0 100" d="M${pts.join('L')}"/>`;
  }
  svg.dataset.v = key;
  const mk = svg.querySelector('.mk');
  requestAnimationFrame(() => mk?.setAttribute('stroke-dasharray', `${Math.max(0, Math.min(100, v)).toFixed(1)} 100`));
}

// the motto, with its bracketed words blacked out (and, pointed at, read through)
function redact(el) {
  if (!el) return;
  const t = el.textContent;
  if (!t.includes('[')) return;
  el.innerHTML = esc(t).replace(/\[([^\]]+)\]/g, '<s>$1</s>');
}

// the evidence tag: the marker writing, each character a touch off the line
function marker(el, text) {
  if (!el || el.dataset.t === text) return;
  el.dataset.t = text;
  el.innerHTML = [...text].map((ch, k) => {
    if (ch === ' ') return '<i class="sp"></i>';
    const h = ((k + 3) * 7919 + ch.charCodeAt(0) * 104729) % 1000 / 1000;
    return `<b style="--r:${(((h * 13) % 1 - 0.5) * 6).toFixed(1)}deg;--y:${(((h * 29) % 1 - 0.5) * 2.4).toFixed(1)}px">${esc(ch)}</b>`;
  }).join('');
}

export default {
  ws: NOTES,
  day: DAY,
  dayLong: DAYS.map(s => s[0] + s.slice(1).toLowerCase()),
  clock: d => ({ hm: d.toTimeString().slice(0, 5), day: `${pad(d.getDate())} ${MON[d.getMonth()]}` }),
  wx: SKY,
  // each window, as it appears in the file
  app: { windowsterminal: 'TELETYPE', zen: 'WITNESS', firefox: 'WITNESS', chrome: 'WITNESS', msedge: 'WITNESS', brave: 'WITNESS',
    opera: 'WITNESS', code: 'FORGERY', cursor: 'FORGERY', explorer: 'RECORDS', files: 'RECORDS', spotify: 'TAPE',
    discord: 'INFORMANT', telegram: 'INFORMANT', claude: 'CONSULTANT', steam: 'ALIBI', notion: 'LEDGER', obsidian: 'LEDGER',
    settings: 'PROCEDURE', photoshop: 'FORGERY', blender: 'RECONSTRUCTION', vlc: 'FOOTAGE', obs64: 'SURVEILLANCE' },
  appWindow: 'SUBJECT', appIdle: 'ON FILE', idle: 'dossier',
  names: { windowsterminal: 'Terminal', msedge: 'Edge', code: 'VS Code', explorer: 'Files', claude: 'Claude', zen: 'Zen',
    chrome: 'Chrome', firefox: 'Firefox', brave: 'Brave', spotify: 'Spotify', discord: 'Discord', steam: 'Steam', obs64: 'OBS' },
  nowm: ['SUSPENDED', 'the file is out on loan'],
  t: { cpu: 'Processor', ram: 'Memory', battery: 'Cell' },

  dash: {
    size: { dash: [880, 540], power: [440, 310], wall: [930, 250], keys: [900, 440] },
    date: d => { const i = +store('rs.ws') || 0; return [d.toTimeString().slice(0, 5), `${DAYS[d.getDay()]} ${d.getDate()} ${MONTH[d.getMonth()]} ${d.getFullYear()}`, `TAB ${pad(i + 1)} · ${tab(i)}`]; },
    month: (y, m) => `${MONTH[m]} ${y}`, wd: ['S', 'M', 'T', 'W', 'T', 'F', 'S'],
    rings: [['cpu', 'PROCESSOR', 'load'], ['ram', 'MEMORY', 'in use'], ['disk', 'ARCHIVE', 'filled'], ['bat', 'CELL', 'remaining']],
    idleMedia: ['NO TAPE', 'the recorder is empty'],
    mottos: MOTTOS,
    power: { lock: ['SEALED', 'Lock'], sleep: ['ON HOLD', 'Sleep'], restart: ['REOPENED', 'Restart'], off: ['CLOSED', 'Shut down'],
      idle: ['FINAL ENTRY', 'How is this file closed?', 'Esc to keep it open'], confirm: 'stamp again to confirm', twice: 'stamp twice' },
    walls: [['TAB 01', 'Evidence'], ['TAB 02', 'Suspects'], ['TAB 03', 'Timeline'], ['TAB 04', 'Reports'], ['TAB 05', 'Closed'], ['ANNEX', 'Yours']],
    wallOn: 'ON THE BOARD', wallBusy: 'DEVELOPING …', wallFail: 'the print came out blank',
    keys: {
      windows: ['§ 1', 'Windows', 'Ctrl + Alt +'], workspaces: ['§ 2', 'The tabs', 'Ctrl + Alt +'], escape: ['§ 3', 'When it goes wrong'],
      bar: ['§ 4', 'The bar', 'mouse'],
      barRows: [['folder', 'overview'], ['tabs', 'go · wheel · right: send'], ['A·1 – A·4', 'Inquiry · Records · Terminal · Codes'],
        ['subject', 'wheel focus · 2× maximize'], ['', 'right: float · middle: close'], ['‹ tray', 'tray icons drawer'],
        ['tape', 'play · right: next · mid: prev'], ['rings', 'Task Manager'], ['cell', 'battery settings'],
        ['volume', 'wheel · click mute · right: sound'], ['date stamp', 'dashboard · right: notifications'], ['seal', 'lock · sleep · restart · off']],
    },
  },

  // the desk widget is an evidence tag tied to the wallpaper, the date and time on it in marker
  desk(d) {
    marker($('dday'), `${DAY[d.getDay()]} ${d.getDate()} ${MON[d.getMonth()]}`);
    marker($('dtime'), d.toTimeString().slice(0, 5));
    set($('dyear'), 'textContent', String(d.getFullYear()));
    if (!$('dtab')?.textContent) this.fx.deskWs(+store('rs.ws') || 0);
  },

  fx: {
    // a tab: its name typed on it; the open one is pulled forward and clipped (bar.css)
    wsButton: n => `<span class="wn">${esc(n)}</span>`,
    ring: (svg, v) => ring(svg, v, false),
    dashRingHTML: (id, k, en) => `<div class="ring2" id="g-${id}"><div class="mkr"><svg viewBox="0 0 64 64" id="r-${id}"></svg><em id="v-${id}">–</em></div>` +
      `<div class="rt"><b>${k}</b><span id="s-${id}">${en}</span></div></div>`,
    dashRing(id, v, reset) { const s = $(`r-${id}`); if (reset && s) { s.dataset.v = ''; s.innerHTML = ''; } ring(s, reset ? 0 : v, true); },
    // the file is pulled from the drawer, then the stamp comes down on it
    dashOpen(i) {
      redact($('precept'));
      set($('artNo'), 'textContent', `${exhibit(i)} · CASE 1105-B`);
      const b = document.body; b.classList.remove('pull'); void b.offsetWidth; b.classList.add('pull');
    },
    deskWs(i) { set($('dtab'), 'textContent', `${exhibit(i)} · ${tab(i)}`); },
    deskWx(k, c) { set($('dwx'), 'textContent', `${SKY[k] ?? SKY.clear} · ${c}°`); },
  },
};
