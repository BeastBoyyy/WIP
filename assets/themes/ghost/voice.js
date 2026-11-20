// Ghost Signal — a HUD seen through rain. Five channels (CH-01 … CH-05), each with its signal meter; every label a
// four-letter code, every figure in the terminal face; what changes flickers once as it locks on.
import { $, set, store, esc } from './engine.js';
import { WS } from './gen.js';

const MON = ['JAN', 'FEB', 'MAR', 'APR', 'MAY', 'JUN', 'JUL', 'AUG', 'SEP', 'OCT', 'NOV', 'DEC'];
const DAY = ['SUN', 'MON', 'TUE', 'WED', 'THU', 'FRI', 'SAT'];
const DAYS = ['SUNDAY', 'MONDAY', 'TUESDAY', 'WEDNESDAY', 'THURSDAY', 'FRIDAY', 'SATURDAY'];
// what each channel shows, and a line of its log
const FEEDS = ['CITY', 'RAIN', 'TERRAIN', 'TOWER', 'GHOST'];
const LOG = ['Towers in the fog; a sign burning down the tallest.', 'The city out of focus beyond wet glass.',
  'A ridge in wireframe, running off to a banded sun.', 'A lattice mast sending its rings across the haze.', 'Someone on a rooftop in the rain. No record.'];
const SKY = { clear: 'CLR', cloudy: 'OVC', light_rain: 'RAIN', heavy_rain: 'STRM', snow: 'SNOW', thunder: 'TSTM' };
const pad = n => String(n).padStart(2, '0');
const ch = i => String(WS[i] ?? `CH-0${i + 1}`);
const doy = d => Math.floor((d - new Date(d.getFullYear(), 0, 1)) / 864e5) + 1;
const week = d => { const t = new Date(Date.UTC(d.getFullYear(), d.getMonth(), d.getDate())); const n = t.getUTCDay() || 7; t.setUTCDate(t.getUTCDate() + 4 - n); return Math.ceil(((t - Date.UTC(t.getUTCFullYear(), 0, 1)) / 864e5 + 1) / 7); };

// a figure that flickers once when it changes, as a readout re-locking (glitch, 90 ms; bar.css)
function tick(el, s) {
  if (!el || el.textContent === s) return;
  el.textContent = s;
  el.classList.remove('tick'); void el.offsetWidth; el.classList.add('tick');
}
// an arc gauge: 3/4 of a circle, ticks every tenth, the reading as a bright arc (redrawn only when it moves a step)
function gauge(svg, v) {
  if (!svg) return;
  const q = Math.round(Math.max(0, Math.min(100, v)) / 2.5);
  if (svg.dataset.v === String(q)) return;
  svg.dataset.v = String(q);
  const R = 26, C = 32, a0 = 135, span = 270, P = a => { const r = (a * Math.PI) / 180; return [(C + R * Math.cos(r)).toFixed(2), (C + R * Math.sin(r)).toFixed(2)]; };
  const arc = (from, to) => { const [x0, y0] = P(from), [x1, y1] = P(to); return `M${x0} ${y0}A${R} ${R} 0 ${to - from > 180 ? 1 : 0} 1 ${x1} ${y1}`; };
  let s = `<path class="trk" d="${arc(a0, a0 + span)}"/>`;
  if (q > 0) s += `<path class="val" d="${arc(a0, a0 + span * q / 40)}"/>`;
  for (let k = 0; k <= 10; k++) { const a = ((a0 + span * k / 10) * Math.PI) / 180; s += `<line class="tk" x1="${(C + 30 * Math.cos(a)).toFixed(2)}" y1="${(C + 30 * Math.sin(a)).toFixed(2)}" x2="${(C + 33 * Math.cos(a)).toFixed(2)}" y2="${(C + 33 * Math.sin(a)).toFixed(2)}"/>`; }
  svg.innerHTML = s;
}
// the processor's radar: rings, a sweep turning (dash.css), and a blip for every tenth of load, scattered but fixed
function radar(svg, v) {
  if (!svg) return;
  const n = Math.round(Math.max(0, Math.min(100, v)) / 10);
  if (svg.dataset.v === String(n)) return;
  svg.dataset.v = String(n);
  let s = '<circle class="rg" cx="32" cy="32" r="30"/><circle class="rg" cx="32" cy="32" r="20"/><circle class="rg" cx="32" cy="32" r="10"/>' +
    '<path class="rg" d="M32 2v60M2 32h60"/><path class="sweep" d="M32 32L32 2A30 30 0 0 1 53.2 10.8z"/>';
  for (let k = 0; k < n; k++) { const a = k * 2.39996, r = 8 + ((k * 7.3) % 21); s += `<circle class="blip" cx="${(32 + r * Math.cos(a)).toFixed(1)}" cy="${(32 + r * Math.sin(a)).toFixed(1)}" r="1.6"/>`; }
  svg.innerHTML = s;
}

export default {
  ws: FEEDS,
  day: DAY,
  dayLong: DAYS,
  clock: d => ({ hm: d.toTimeString().slice(0, 5), day: `${DAY[d.getDay()]} ${pad(d.getDate())}.${pad(d.getMonth() + 1)}` }),
  wx: SKY,
  // each program, as a HUD code
  app: { windowsterminal: 'TERM', zen: 'NET', firefox: 'NET', chrome: 'NET', msedge: 'NET', brave: 'NET', opera: 'NET',
    code: 'CODE', cursor: 'CODE', explorer: 'FILE', files: 'FILE', spotify: 'AUDIO', discord: 'COMM', telegram: 'COMM', claude: 'AI',
    steam: 'GAME', notion: 'NOTE', obsidian: 'NOTE', settings: 'SYS', photoshop: 'IMG', blender: 'MESH', vlc: 'VID', obs64: 'REC' },
  appWindow: 'PROC', appIdle: 'IDLE', idle: 'no target',
  names: { windowsterminal: 'Terminal', msedge: 'Edge', code: 'VS Code', explorer: 'Files', claude: 'Claude', zen: 'Zen',
    chrome: 'Chrome', firefox: 'Firefox', brave: 'Brave', spotify: 'Spotify', discord: 'Discord', steam: 'Steam', obs64: 'OBS' },
  nowm: ['NO SIGNAL', 'no window on this channel'],
  t: { cpu: 'CPU', ram: 'MEM', battery: 'PWR' },

  dash: {
    size: { dash: [900, 540], power: [440, 320], wall: [930, 250], keys: [900, 440] },
    closeMs: 120,
    // to the second while it is open; the third line is the day and week of the year
    date: d => [d.toTimeString().slice(0, 8), `${DAYS[d.getDay()]} ${pad(d.getDate())} ${MON[d.getMonth()]} ${d.getFullYear()}`, `DAY ${doy(d)} // WK ${week(d)}`],
    month: (y, m) => `${MON[m]} ${y}`, wd: ['SU', 'MO', 'TU', 'WE', 'TH', 'FR', 'SA'],
    num: n => pad(n),
    rings: [['cpu', 'CPU', 'load'], ['ram', 'MEM', 'in use'], ['disk', 'DSK', 'full'], ['bat', 'PWR', 'cell']],
    idleMedia: ['NO CARRIER', 'audio channel silent'],
    mottos: ['SIGNAL IS NOT NOISE.', 'STAY ON THE FREQUENCY.', 'EVERY CITY HAS A GHOST.', 'LOCK ON. HOLD STILL.',
      'THE RAIN REMEMBERS.', 'NO RECORD IS STILL A RECORD.', 'WATCH THE STATIC.', 'TRUST THE CARRIER.'],
    power: { lock: ['LOCK', 'Seal the console'], sleep: ['SLEEP', 'Drop to standby'], restart: ['REBOOT', 'Cycle the system'],
      off: ['HALT', 'Cut all power'], idle: ['STANDBY', 'Select an action', 'ESC // ABORT'], confirm: 'CONFIRM: PRESS AGAIN', twice: 'DOUBLE PRESS' },
    walls: [['CH-01', 'CITY'], ['CH-02', 'RAIN'], ['CH-03', 'TERRAIN'], ['CH-04', 'TOWER'], ['CH-05', 'GHOST'], ['EXT', 'YOURS']],
    wallOn: 'LIVE', wallBusy: 'TUNING …', wallFail: 'NO CARRIER',
    keys: {
      windows: ['01', 'WINDOWS', 'Ctrl + Alt +'], workspaces: ['02', 'CHANNELS', 'Ctrl + Alt +'], escape: ['03', 'ABORT'],
      bar: ['04', 'THE STRIP', 'pointer'],
      barRows: [['reticle', 'overview'], ['CH-01 … 05', 'go · wheel · right: send'], ['SRCH · FILE · TERM · KEYS', 'Flow · Explorer · Terminal · this'],
        ['target', 'wheel focus · 2× maximize'], ['', 'right: float · middle: close'], ['‹ tray', 'tray icons drawer'],
        ['AUD', 'play · right: next · mid: prev'], ['CPU · MEM', 'Task Manager'], ['PWR cell', 'battery settings'],
        ['VOL', 'wheel · click mute · right: sound'], ['time', 'dashboard · right: notifications'], ['PWR', 'lock · sleep · restart · off']],
    },
  },

  // the desk readout: the date as coordinates, the time to the second (its own 1 s timer, held while a game is on),
  // and a heartbeat trace that beats once as each minute turns
  desk(d) {
    set($('dco'), 'textContent', `${d.getFullYear()}°${pad(d.getMonth() + 1)}′${pad(d.getDate())}″ N  //  HDG ${DAY[d.getDay()]}  //  DAY ${doy(d)}`);
    const b = $('beat'); if (b) { b.classList.remove('go'); void b.getBoundingClientRect(); b.classList.add('go'); }
    if (!$('dws')?.textContent) this.fx.deskWs(+store('rs.ws') || 0);
    if (!this._sec) {
      const run = () => {
        const n = new Date();
        if (!document.body.classList.contains('nowm')) { set($('dhm'), 'textContent', n.toTimeString().slice(0, 5)); set($('dss'), 'textContent', pad(n.getSeconds())); }
        this._sec = setTimeout(run, 1000 - n.getMilliseconds() + 5);
      };
      run();
    }
  },

  fx: {
    // a channel: its code, and a meter of five bars (bar.css lights them: all for the open one, three for one in use)
    wsButton: (n, i) => `<span class="cn">${esc(n)}</span><i class="sig"><i></i><i></i><i></i><i></i><i></i></i>`,
    ring: (el, v) => tick(el, pad(Math.round(Math.max(0, Math.min(99, v))))),
    // changing channel sends the scan line across the strip once (bar.css); otherwise it stays still
    onActive: (b, i, last, reduced) => { const s = $('bar'); if (!s || reduced) return; s.classList.remove('scan'); void s.offsetWidth; s.classList.add('scan'); },
    // the processor gets the radar; the rest arc gauges
    dashRingHTML: (id, k, en) => `<div class="ring2" id="g-${id}"><div class="dial"><svg viewBox="0 0 64 64" id="r-${id}"></svg><em id="v-${id}">–</em></div>` +
      `<div class="rt"><b>${k}</b><span id="s-${id}">${en}</span></div></div>`,
    dashRing(id, v, reset) { const s = $(`r-${id}`); if (reset && s) s.dataset.v = ''; (id === 'cpu' ? radar : gauge)(s, reset ? 0 : v); },
    // the feed full size (the halftone dots blur when the small card is blown up), with its log line and channel
    dashOpen(i) {
      set($('artImg'), 'src', new URL(`./art/wall-${i % 5}.jpg`, location.href).href);
      set($('log'), 'textContent', LOG[i % 5]);
      set($('hdr'), 'textContent', `GHOST SIGNAL // ${ch(i)} // ${FEEDS[i % 5]}`);
    },
    deskWs(i) { set($('dws'), 'textContent', `${ch(i)} // ${FEEDS[i % 5]}`); },
    deskWx(k, c) { set($('dwx'), 'textContent', `${SKY[k] ?? SKY.clear} ${c}°C`); },
  },
};
