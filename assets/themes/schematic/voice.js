// Schematic — the drawing office: sheets of one set for rooms, numbered callouts for verbs, a title block for a bar.
import { $, set, store } from './engine.js';
import { WS } from './gen.js';

const MON = ['JAN', 'FEB', 'MAR', 'APR', 'MAY', 'JUN', 'JUL', 'AUG', 'SEP', 'OCT', 'NOV', 'DEC'];
const DAYS = ['SUNDAY', 'MONDAY', 'TUESDAY', 'WEDNESDAY', 'THURSDAY', 'FRIDAY', 'SATURDAY'];
const DAY = ['SUN', 'MON', 'TUE', 'WED', 'THU', 'FRI', 'SAT'];
const pad = n => String(n).padStart(2, '0');
const sheet = i => `A-${pad(i + 1)}`;
const name = i => String(WS[i] ?? i + 1).toUpperCase();
// the sky, as the site diary has it
const SKY = { clear: 'CLEAR', cloudy: 'OVERCAST', light_rain: 'DRIZZLE', heavy_rain: 'RAIN', snow: 'SNOW', thunder: 'STORM' };
const NOTES = ['Do not scale from drawing', 'All dimensions to be verified on site', 'Measure twice, cut once', 'Flanks before faces',
  'Hold the high ground', 'Tolerance ±0.5 unless noted', 'Check it, then check it again', 'Every line has a reason'];

// a scale rule: ticks every tenth, the run filled to the value in construction blue, a yellow pointer riding on top
function rule(svg, v, figures) {
  if (!svg) return;
  const key = String(Math.round(v));
  if (svg.dataset.v === key) return;
  svg.dataset.v = key;
  const vb = svg.viewBox.baseVal, x0 = 3, x1 = vb.width - 3, y = vb.height - (figures ? 10 : 2);
  const at = x0 + (x1 - x0) * Math.max(0, Math.min(100, v)) / 100;
  svg.style.setProperty('--v', (v / 100).toFixed(3));
  const fill = svg.querySelector('.fill'), ptr = svg.querySelector('.ptr');
  if (fill && ptr) { fill.setAttribute('width', String(at - x0)); ptr.style.transform = `translateX(${at}px)`; return; }
  const big = figures ? 2 : 1;
  let marks = '';
  for (let k = 0; k <= 10; k++) {
    const x = x0 + (x1 - x0) * k / 10, l = (k % 5 ? 2.5 : 4.5) * big;
    marks += `<line class="tick" x1="${x}" y1="${y}" x2="${x}" y2="${y - l}"/>`;
    if (figures && k % 5 === 0) marks += `<text x="${x}" y="${y + 9}">${k * 10}</text>`;
  }
  svg.innerHTML = `<rect class="fill" x="${x0}" y="${y - 1.2 * big}" width="${at - x0}" height="${1.2 * big}"/>` +
    `<line class="base" x1="${x0}" y1="${y}" x2="${x1}" y2="${y}"/>${marks}` +
    `<path class="ptr" style="transform:translateX(${at}px)" d="M0 ${y - 5 * big}l${-2.6 * big} ${-4 * big}h${5.2 * big}z"/>`;
}

// the clock is a protractor: the minute needle sweeps the half-circle once an hour, the hour rides the rim in yellow
function dial(d) {
  const m = d.getMinutes() + d.getSeconds() / 60, h = (d.getHours() % 12) + m / 60;
  $('pmin')?.style.setProperty('transform', `rotate(${m * 3}deg)`);
  $('phr')?.style.setProperty('transform', `rotate(${h * 15}deg)`);
}
function protractor(svg) {
  if (!svg || svg.childElementCount) return;
  const cx = 150, cy = 146, r = 128;
  let s = `<path class="arc" d="M${cx - r} ${cy}A${r} ${r} 0 0 1 ${cx + r} ${cy}"/><path class="arc in" d="M${cx - 64} ${cy}A64 64 0 0 1 ${cx + 64} ${cy}"/>` +
    `<line class="base" x1="${cx - r - 8}" y1="${cy}" x2="${cx + r + 8}" y2="${cy}"/>`;
  for (let k = 0; k <= 60; k++) {
    const a = Math.PI + k * Math.PI / 60, l = k % 15 === 0 ? 14 : k % 5 === 0 ? 9 : 5;
    const [c, si] = [Math.cos(a), Math.sin(a)];
    s += `<line class="${k % 5 ? 'tick' : 'tick big'}" x1="${cx + c * r}" y1="${cy + si * r}" x2="${cx + c * (r - l)}" y2="${cy + si * (r - l)}"/>`;
    if (k % 5 === 0) s += `<text class="${k % 15 ? '' : 'q'}" x="${cx + c * (r - 22)}" y="${cy + si * (r - 22) + 3.5}">${pad(k)}</text>`;
  }
  s += `<g id="phr" style="transform-origin:${cx}px ${cy}px"><path class="hr" d="M${cx - r - 2} ${cy}l-9 -5v10z"/></g>` +
    `<g id="pmin" style="transform-origin:${cx}px ${cy}px"><line class="needle" x1="${cx - 70}" y1="${cy}" x2="${cx - r + 2}" y2="${cy}"/><circle class="tip" cx="${cx - r + 2}" cy="${cy}" r="2.6"/></g>` +
    `<path class="cross" d="M${cx - 7} ${cy}h14M${cx} ${cy - 7}v7"/>`;
  svg.innerHTML = s;
  dial(new Date());
}

export default {
  // what each sheet shows, under its name (the dash caption, the tooltip, the title when nothing is open)
  ws: ['the field from above', 'a cut through the walls', 'the engine from the side', 'every part, numbered', 'notes and the trace'],
  day: DAY,
  dayLong: DAYS.map(s => s[0] + s.slice(1).toLowerCase()),
  clock: d => ({ hm: d.toTimeString().slice(0, 5), day: `${DAY[d.getDay()]} ${pad(d.getDate())}.${pad(d.getMonth() + 1)}` }),
  wx: SKY,
  // what each window is, as a part on the drawing
  app: { windowsterminal: 'CONSOLE', zen: 'VIEWER', firefox: 'VIEWER', chrome: 'VIEWER', msedge: 'VIEWER', brave: 'VIEWER', opera: 'VIEWER',
    code: 'CAD', cursor: 'CAD', explorer: 'FILES', files: 'FILES', spotify: 'AUDIO', discord: 'COMMS', telegram: 'COMMS',
    claude: 'ADVISOR', steam: 'FIELD', notion: 'LOG', obsidian: 'LOG', settings: 'CONFIG', photoshop: 'RENDER', blender: 'MODEL',
    vlc: 'OPTICS', obs64: 'OPTICS' },
  appWindow: 'PART', appIdle: 'BLANK', idle: 'schematic',
  names: { windowsterminal: 'Terminal', msedge: 'Edge', code: 'VS Code', explorer: 'Files', claude: 'Claude', zen: 'Zen',
    chrome: 'Chrome', firefox: 'Firefox', brave: 'Brave', spotify: 'Spotify', discord: 'Discord', steam: 'Steam', obs64: 'OBS' },
  nowm: ['ON HOLD', 'the office is on hold'],
  t: { cpu: 'Processor', ram: 'Memory', battery: 'Cell' },

  dash: {
    size: { dash: [880, 540], power: [420, 300], wall: [930, 240], keys: [900, 430] },
    date: d => { dial(d); const i = +store('rs.ws') || 0; return [d.toTimeString().slice(0, 5), `${DAYS[d.getDay()]} ${pad(d.getDate())} ${MON[d.getMonth()]} ${d.getFullYear()}`, `SHEET ${sheet(i)} · ${name(i)}`]; },
    month: (y, m) => `${MON[m]} ${y}`, wd: ['S', 'M', 'T', 'W', 'T', 'F', 'S'],
    rings: [['cpu', 'PROCESSOR', 'load'], ['ram', 'MEMORY', 'in use'], ['disk', 'STORAGE', 'filled'], ['bat', 'CELL', 'charge']],
    idleMedia: ['PLOTTER IDLE', 'nothing on the pen'],
    mottos: NOTES,
    power: { lock: ['HOLD', 'Lock'], sleep: ['SHELVE', 'Sleep'], restart: ['REISSUE', 'Restart'], off: ['ARCHIVE', 'Shut down'],
      idle: ['REVISION', 'Issue the drawing', 'Esc to keep drafting'], confirm: 'stamp again to confirm', twice: 'stamp twice' },
    walls: [['A-01', 'Plan'], ['A-02', 'Section'], ['A-03', 'Profile'], ['A-04', 'Detail'], ['A-05', 'Notes'], ['—', 'Yours']],
    wallOn: 'ISSUED', wallBusy: 'PLOTTING …', wallFail: 'the plotter jammed',
    keys: {
      windows: ['PARTS', 'Windows', 'Ctrl + Alt +'], workspaces: ['SHEETS', 'The set', 'Ctrl + Alt +'], escape: ['VOID', 'When it goes wrong'],
      bar: ['TITLE BLOCK', 'The bar', 'mouse'],
      barRows: [['target', 'overview'], ['sheets', 'go · wheel · right: send'], ['callouts', 'Search · Files · Console · Legend'],
        ['part', 'wheel focus · 2× maximize'], ['', 'right: float · middle: close'], ['‹ tray', 'tray icons drawer'],
        ['plotter', 'play · right: next · mid: prev'], ['load', 'Task Manager'], ['cell', 'battery settings'],
        ['volume', 'wheel · click mute · right: sound'], ['time', 'dashboard · right: notifications'], ['stamp', 'lock · sleep · restart · off']],
    },
  },

  // the desk widget is a title block in the corner of the wallpaper; the minute is re-plotted each time it turns
  desk(d) {
    set($('dh'), 'textContent', pad(d.getHours()));
    set($('dm'), 'textContent', pad(d.getMinutes()));
    const s = $('dscale');
    if (s) { s.classList.remove('plot'); void s.offsetWidth; s.classList.add('plot'); }
    set($('dday'), 'textContent', DAYS[d.getDay()]);
    set($('ddate'), 'textContent', `${pad(d.getDate())} ${MON[d.getMonth()]} ${d.getFullYear()}`);
    if (!$('dsheet')?.textContent) this.fx.deskWs(+store('rs.ws') || 0);
  },

  fx: {
    // a sheet: its number over its name; the one open is traced round by the plotter's pen
    wsButton: (n, i) => `<span class="no">${sheet(i)}</span><span class="wn">${n}</span>` +
      `<svg class="plot" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true"><rect x="1" y="1" width="98" height="98" pathLength="100" vector-effect="non-scaling-stroke"/></svg>`,
    ring: (svg, v) => rule(svg, v, false),
    dashRingHTML: (id, k, en) => `<div class="ring2" id="g-${id}"><div class="rh"><b>${k}</b><em id="v-${id}">–</em></div>` +
      `<svg viewBox="0 0 150 30" id="r-${id}"></svg><span id="s-${id}">${en}</span></div>`,
    dashRing(id, v, reset) { const s = $(`r-${id}`); if (reset && s) s.dataset.v = ''; rule(s, reset ? 0 : v, true); },
    // the sheet comes out of the drawer folded in four and opens along its creases
    dashOpen() { const b = document.body; b.classList.remove('unfold'); void b.offsetWidth; b.classList.add('unfold'); },
    dashReady() { protractor($('prot')); },
    deskWs(i) { set($('dsheet'), 'textContent', `${sheet(i)} · ${name(i)}`); },
    deskWx(k, c) { set($('dwx'), 'textContent', `${SKY[k] ?? SKY.clear} ${c}°`); },
  },
};
