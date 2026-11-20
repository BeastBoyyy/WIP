// Etch — five copper plates, printed black, one point of light in each. Labels engraved small with a fleuron between,
// the titles in Latin as on an old print, the numbers in roman.
import { $, set, store, esc } from './engine.js';
import { WS } from './gen.js';

const MON = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
const MONTH = ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December'];
const DAYS = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];
const DAY = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
const ROMAN = ['I', 'II', 'III', 'IV', 'V', 'VI', 'VII', 'VIII', 'IX', 'X', 'XI', 'XII'];
// what each plate shows
const TITLES = ['Lux in tenebris', 'Pharos', 'Luna', 'Memento mori', 'Aurora'];
const LEGEND = ['A candle on the table, a book, a quill standing in its well.', 'The lighthouse on the headland, its beam laid across the sea.',
  'The full moon over the hills, and the river carrying its light.', 'A skull on a closed book; the hourglass all but run.',
  'The sun in the notch of the mountains, the valley waking.'];
const plate = i => String(WS[i] ?? `Plate ${ROMAN[i] ?? i + 1}`);
const SKY = { clear: 'fair', cloudy: 'overcast', light_rain: 'showers', heavy_rain: 'tempest', snow: 'snow', thunder: 'thunder' };
const ord = n => n + (n % 100 > 10 && n % 100 < 14 ? 'th' : ['th', 'st', 'nd', 'rd'][n % 10] || 'th');
const roman = n => { const v = [[10, 'x'], [9, 'ix'], [5, 'v'], [4, 'iv'], [1, 'i']]; let s = ''; for (const [k, r] of v) while (n >= k) { s += r; n -= k; } return s; };

// a dial engraved on an instrument: `n` ticks round it, those up to the reading cut bright, the rest faint
// (rebuilt only when the count moves)
function dial(svg, v, n) {
  if (!svg) return;
  const on = Math.round(Math.max(0, Math.min(100, v)) / 100 * n);
  if (svg.dataset.v === String(on)) return;
  svg.dataset.v = String(on);
  const vb = svg.viewBox.baseVal, c = vb.width / 2, r2 = c * 0.94, r1 = c * (n > 20 ? 0.74 : 0.6);
  let s = `<circle class="rim" cx="${c}" cy="${c}" r="${(c * 0.97).toFixed(2)}"/>`;
  for (let k = 0; k < n; k++) {
    const a = k / n * Math.PI * 2 - Math.PI / 2, cs = Math.cos(a), sn = Math.sin(a), rr = k % 5 === 0 && n > 20 ? r1 - c * 0.08 : r1;
    s += `<line class="${k < on ? 'on' : 'off'}" x1="${(c + cs * rr).toFixed(2)}" y1="${(c + sn * rr).toFixed(2)}" x2="${(c + cs * r2).toFixed(2)}" y2="${(c + sn * r2).toFixed(2)}"/>`;
  }
  svg.innerHTML = s;
}

export default {
  ws: TITLES,
  day: DAY,
  dayLong: DAYS,
  clock: d => ({ hm: d.toTimeString().slice(0, 5), day: `${DAY[d.getDay()]} · ${roman(d.getDate())} · ${MON[d.getMonth()]}` }),
  wx: SKY,
  // each program, as a room in the engraver's house
  app: { windowsterminal: 'THE PRESS', zen: 'THE ATLAS', firefox: 'THE ATLAS', chrome: 'THE ATLAS', msedge: 'THE ATLAS', brave: 'THE ATLAS',
    opera: 'THE ATLAS', code: 'THE WORKSHOP', cursor: 'THE WORKSHOP', explorer: 'THE CABINET', files: 'THE CABINET', spotify: 'THE ORGAN',
    discord: 'THE SALON', telegram: 'THE SALON', claude: 'THE ORACLE', steam: 'THE GAMING ROOM', notion: 'COMMONPLACE', obsidian: 'COMMONPLACE',
    settings: 'THE CLOCKWORK', photoshop: 'THE STUDIO', blender: 'THE FOUNDRY', vlc: 'THE THEATRE', obs64: 'CAMERA OBSCURA' },
  appWindow: 'AT WORK', appIdle: 'TABULA RASA', idle: 'etch',
  names: { windowsterminal: 'Terminal', msedge: 'Edge', code: 'VS Code', explorer: 'Files', claude: 'Claude', zen: 'Zen',
    chrome: 'Chrome', firefox: 'Firefox', brave: 'Brave', spotify: 'Spotify', discord: 'Discord', steam: 'Steam', obs64: 'OBS' },
  nowm: ['THE PRESS IS STILL', 'nothing is being printed'],
  t: { cpu: 'Processor', ram: 'Memory', battery: 'Battery' },

  dash: {
    size: { dash: [900, 540], power: [440, 320], wall: [930, 250], keys: [900, 440] },
    closeMs: 160,
    date: d => { const i = +store('rs.ws') || 0; return [d.toTimeString().slice(0, 5), `${DAYS[d.getDay()]}, the ${ord(d.getDate())} of ${MONTH[d.getMonth()]}`, `${plate(i)} · ${TITLES[i % 5]}`]; },
    month: (y, m) => `${MONTH[m]} ${y}`, wd: ['S', 'M', 'T', 'W', 'T', 'F', 'S'],
    rings: [['cpu', 'Processor', 'labouring'], ['ram', 'Memory', 'held'], ['disk', 'Library', 'shelved'], ['bat', 'Cell', 'remaining']],
    idleMedia: ['Silence', 'the organ is at rest'],
    mottos: ['Lux in tenebris.', 'Nulla dies sine linea.', 'Festina lente.', 'Per aspera ad astra.', 'Ars longa, vita brevis.',
      'Dum spiro, spero.', 'Sapere aude.', 'Memento mori.'],
    power: { lock: ['SEALED', 'Lock'], sleep: ['NOCTURNE', 'Sleep'], restart: ['REPRISE', 'Restart'], off: ['FINIS', 'Shut down'],
      idle: ['EXPLICIT', 'How shall the plate be finished?', 'Esc to return to the work'], confirm: 'press again to strike it', twice: 'press twice' },
    walls: [['Plate I', 'Lux in tenebris'], ['Plate II', 'Pharos'], ['Plate III', 'Luna'], ['Plate IV', 'Memento mori'], ['Plate V', 'Aurora'], ['Ex libris', 'Yours']],
    wallOn: 'PRINTED', wallBusy: 'inking the plate …', wallFail: 'the plate slipped',
    keys: {
      windows: ['I', 'Windows', 'Ctrl + Alt +'], workspaces: ['II', 'Plates', 'Ctrl + Alt +'], escape: ['III', 'In extremis'],
      bar: ['IV', 'The bar', 'by the mouse'],
      barRows: [['frame', 'overview'], ['numerals', 'go · wheel · right: send'], ['seek · folio · press · keys', 'Flow · Explorer · Terminal · this'],
        ['the title', 'wheel focus · 2× maximize'], ['', 'right: float · middle: close'], ['‹ tray', 'tray icons drawer'],
        ['organ', 'play · right: next · mid: prev'], ['dials', 'Task Manager'], ['cell', 'battery settings'],
        ['volume', 'wheel · click mute · right: sound'], ['clock', 'dashboard · right: notifications'], ['the light', 'lock · sleep · restart · off']],
    },
  },

  // the desk widget is a pocket watch, engraved: the hands set once a minute, the day on its cartouche
  desk(d) {
    const h = d.getHours() % 12, m = d.getMinutes();
    $('hh')?.style.setProperty('--a', `${(h + m / 60) * 30}deg`);
    $('mh')?.style.setProperty('--a', `${m * 6}deg`);
    set($('dday'), 'textContent', `${DAYS[d.getDay()]} · ${roman(d.getDate())} · ${MONTH[d.getMonth()]}`);
    if (!$('dno')?.textContent) this.fx.deskWs(+store('rs.ws') || 0);
  },

  fx: {
    // a plate: its numeral alone, cut in the display face; the open one is the lit point (bar.css)
    wsButton: n => { const m = String(n).match(/^(plate\s+)(.+)$/i); return `<b>${esc(m ? m[2] : n)}</b><i class="lux"></i>`; },
    ring: (svg, v) => dial(svg, v, 12),
    dashRingHTML: (id, k, en) => `<div class="ring2" id="g-${id}"><div class="dial"><svg viewBox="0 0 64 64" id="r-${id}"></svg><em id="v-${id}">–</em></div>` +
      `<div class="rt"><b>${k}</b><span id="s-${id}">${en}</span></div></div>`,
    dashRing(id, v, reset) { const s = $(`r-${id}`); if (reset && s) s.dataset.v = ''; dial(s, reset ? 0 : v, 40); },
    // the print is pulled from the full plate (the small card would blur its lines), with the plate's legend under it
    dashOpen(i) { set($('artImg'), 'src', new URL(`./art/wall-${i % 5}.jpg`, location.href).href); set($('legend'), 'textContent', LEGEND[i % 5]); },
    deskWs(i) { set($('dno'), 'textContent', `${plate(i)} · ${TITLES[i % 5]}`); },
    deskWx(k, c) { set($('dwx'), 'textContent', `${SKY[k] ?? SKY.clear}, ${c}°`); },
  },
};
