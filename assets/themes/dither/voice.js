// Dither — a 1-bit screen. Five icons drawn on 8×8 grids (sun, moon, star, eye, door), each the name of a picture;
// every label a short word in capitals, every number in terminal figures.
import { $, set, store, esc } from './engine.js';
import { WS } from './gen.js';

const MON = ['JAN', 'FEB', 'MAR', 'APR', 'MAY', 'JUN', 'JUL', 'AUG', 'SEP', 'OCT', 'NOV', 'DEC'];
const MONTH = ['JANUARY', 'FEBRUARY', 'MARCH', 'APRIL', 'MAY', 'JUNE', 'JULY', 'AUGUST', 'SEPTEMBER', 'OCTOBER', 'NOVEMBER', 'DECEMBER'];
const DAYS = ['SUNDAY', 'MONDAY', 'TUESDAY', 'WEDNESDAY', 'THURSDAY', 'FRIDAY', 'SATURDAY'];
const DAY = ['SUN', 'MON', 'TUE', 'WED', 'THU', 'FRI', 'SAT'];
// the picture each icon opens, as its file
const PICS = ['BEAMS', 'CLOUDS', 'SPIRAL', 'WINDOW', 'DOOR'];
const ABOUT = ['The sun behind a bank of cloud, its beams on the sea.', 'A full moon in heaped cloud; one window lit on the ridge.',
  'A spiral of stars turning round its bright core.', 'A rose window, its light laid across the floor.', 'A door alone on the plain, open, and someone before it.'];
// the five icons, the same grids the wallpapers carry: sun, moon, star, eye, door
const ICONS = [
  ['...#....', '.#.#.#..', '..###...', '#######.', '..###...', '.#.#.#..', '...#....', '........'],
  ['..###...', '.##.....', '##......', '##......', '##......', '.##.....', '..###...', '........'],
  ['...#....', '...#....', '..###...', '#######.', '.#####..', '.##.##..', '#.....#.', '........'],
  ['........', '..####..', '.#....#.', '#..##..#', '#..##..#', '.#....#.', '..####..', '........'],
  ['.######.', '.#....#.', '.#....#.', '.#....#.', '.#..#.#.', '.#....#.', '.#....#.', '########'],
];
const dots = rows => rows.map((r, y) => [...r].map((c, x) => c === '#' ? `M${x} ${y}h1v1h-1z` : '').join('')).join('');
const icon = (i, z = 2) => `<svg class="px" width="${8 * z}" height="${8 * z}" viewBox="0 0 8 8" shape-rendering="crispEdges" fill="currentColor"><path d="${dots(ICONS[i % 5])}"/></svg>`;
const SKY = { clear: 'CLEAR', cloudy: 'CLOUDY', light_rain: 'RAIN', heavy_rain: 'STORM', snow: 'SNOW', thunder: 'THUNDER' };
const pic = i => `${String(WS[i] ?? '').toUpperCase()} · ${PICS[i % 5]}.PIC`;

// a bar of the screen's kind: solid as far as the reading, dotted after, in twelfths so it only redraws when it moves
const bar = (el, v) => { if (el) el.style.setProperty('--p', String(Math.round(Math.max(0, Math.min(100, v)) / 100 * 12) / 12)); };

// the desk clock's figures, five dots by seven, and the colon
const FIG = {
  0: ['01110', '10001', '10011', '10101', '11001', '10001', '01110'], 1: ['00100', '01100', '00100', '00100', '00100', '00100', '01110'],
  2: ['01110', '10001', '00001', '00010', '00100', '01000', '11111'], 3: ['11111', '00010', '00100', '00010', '00001', '10001', '01110'],
  4: ['00010', '00110', '01010', '10010', '11111', '00010', '00010'], 5: ['11111', '10000', '11110', '00001', '00001', '10001', '01110'],
  6: ['00110', '01000', '10000', '11110', '10001', '10001', '01110'], 7: ['11111', '00001', '00010', '00100', '01000', '01000', '01000'],
  8: ['01110', '10001', '10001', '01110', '10001', '10001', '01110'], 9: ['01110', '10001', '10001', '01111', '00001', '00010', '01100'],
  ':': ['0', '0', '1', '0', '1', '0', '0'],
};
// the hour as dots on a grid, one <i> a dot, placed by their column and row (the shadow is css)
function figures(s) {
  let x = 0, h = '';
  for (const ch of s) {
    const g = FIG[ch]; if (!g) continue;
    g.forEach((r, y) => [...r].forEach((b, k) => { if (b === '1') h += `<i style="--x:${x + k};--y:${y}"></i>`; }));
    x += g[0].length + 1;
  }
  return [h, x - 1];
}

export default {
  ws: PICS,
  day: DAY,
  dayLong: DAYS,
  clock: d => ({ hm: d.toTimeString().slice(0, 5), day: `${DAY[d.getDay()]} ${d.getDate()} ${MON[d.getMonth()]}` }),
  wx: SKY,
  // each program, as one short word
  app: { windowsterminal: 'SHELL', zen: 'WEB', firefox: 'WEB', chrome: 'WEB', msedge: 'WEB', brave: 'WEB', opera: 'WEB',
    code: 'EDIT', cursor: 'EDIT', explorer: 'DISK', files: 'DISK', spotify: 'SOUND', discord: 'CHAT', telegram: 'CHAT', claude: 'ASK',
    steam: 'GAME', notion: 'NOTES', obsidian: 'NOTES', settings: 'PREFS', photoshop: 'PAINT', blender: 'MODEL', vlc: 'VIDEO', obs64: 'REC' },
  appWindow: 'APP', appIdle: 'DESKTOP', idle: 'dither',
  names: { windowsterminal: 'Terminal', msedge: 'Edge', code: 'VS Code', explorer: 'Files', claude: 'Claude', zen: 'Zen',
    chrome: 'Chrome', firefox: 'Firefox', brave: 'Brave', spotify: 'Spotify', discord: 'Discord', steam: 'Steam', obs64: 'OBS' },
  nowm: ['NO WINDOWS', 'the screen is empty'],
  t: { cpu: 'CPU', ram: 'RAM', battery: 'BATTERY' },

  dash: {
    size: { dash: [900, 540], power: [440, 320], wall: [930, 250], keys: [900, 440] },
    closeMs: 250,
    // the third line is the day's number in the year, out of the year's days
    date: d => { const y = d.getFullYear(), n = Math.floor((d - new Date(y, 0, 1)) / 864e5) + 1, all = (y % 4 || (y % 100 === 0 && y % 400)) ? 365 : 366;
      return [d.toTimeString().slice(0, 5), `${DAYS[d.getDay()]} ${d.getDate()} ${MONTH[d.getMonth()]}`, `DAY ${n}/${all}`]; },
    month: (y, m) => `${MONTH[m]} ${y}`, wd: ['S', 'M', 'T', 'W', 'T', 'F', 'S'],
    rings: [['cpu', 'CPU', 'busy'], ['ram', 'RAM', 'in use'], ['disk', 'DISK', 'full'], ['bat', 'BATT', 'left']],
    idleMedia: ['NO SOUND', 'nothing is playing'],
    mottos: ['ONE BIT IS ENOUGH.', 'NOTHING IS GREY UP CLOSE.', 'LIGHT IS ONLY MORE DOTS.', 'THE PATTERN KNOWS THE TONE.',
      'BLACK, WHITE, AND THE ORDER BETWEEN.', 'EIGHT BY EIGHT IS A WORLD.', 'SAVE EARLY. SAVE OFTEN.', 'EVERY DOT IS A DECISION.'],
    power: { lock: ['LOCK', 'Lock the screen.'], sleep: ['SLEEP', 'Put the machine to sleep.'], restart: ['RESTART', 'Start it again.'],
      off: ['SHUT DOWN', 'Turn it off.'], idle: ['SPECIAL', 'What should the machine do?', 'ESC TO CANCEL'], confirm: 'CLICK AGAIN TO CONFIRM', twice: 'CLICK TWICE' },
    walls: [['SUN', 'BEAMS'], ['MOON', 'CLOUDS'], ['STAR', 'SPIRAL'], ['EYE', 'WINDOW'], ['DOOR', 'DOOR'], ['YOU', 'YOURS']],
    wallOn: 'ON', wallBusy: 'DRAWING…', wallFail: 'DISK ERROR',
    keys: {
      windows: ['1', 'WINDOWS', 'Ctrl + Alt +'], workspaces: ['2', 'SCREENS', 'Ctrl + Alt +'], escape: ['3', 'PANIC'],
      bar: ['4', 'THE BAR', 'by the mouse'],
      barRows: [['close box', 'overview'], ['icons', 'go · wheel · right: send'], ['FIND · DISK · SHELL · KEYS', 'Flow · Explorer · Terminal · this'],
        ['the title', 'wheel focus · 2× maximize'], ['', 'right: float · middle: close'], ['‹ tray', 'tray icons drawer'],
        ['note', 'play · right: next · mid: prev'], ['bars', 'Task Manager'], ['battery', 'battery settings'],
        ['volume', 'wheel · click mute · right: sound'], ['clock', 'dashboard · right: notifications'], ['power', 'lock · sleep · restart · off']],
    },
  },

  // the desk widget is the hour in big dots, set once a minute, with a status line under it
  desk(d) {
    const [h, w] = figures(d.toTimeString().slice(0, 5));
    const el = $('pix');
    if (el) { el.style.setProperty('--w', w); el.innerHTML = h; }
    set($('dday'), 'textContent', `${DAYS[d.getDay()]} ${d.getDate()} ${MON[d.getMonth()]}`);
    if (!$('dws')?.textContent) this.fx.deskWs(+store('rs.ws') || 0);
  },

  fx: {
    wsButton: (n, i) => icon(i),
    ring: bar,
    dashRingHTML: (id, k, en) => `<div class="ring2" id="g-${id}"><b class="k">${k}</b><i class="cb" id="r-${id}"><i></i></i>` +
      `<em id="v-${id}">–</em><span id="s-${id}">${en}</span></div>`,
    // reset empties the bar; the reading then fills it in steps (dash.css)
    dashRing: (id, v, reset) => bar($(`r-${id}`), reset ? 0 : v),
    // the picture is the full-size print shown at a quarter, one screen dot to one pixel (the card would blur them)
    dashOpen(i) {
      set($('artImg'), 'src', new URL(`./art/wall-${i % 5}.jpg`, location.href).href);
      set($('about'), 'textContent', ABOUT[i % 5]);
      set($('wtitle'), 'textContent', pic(i));
      const a = $('aicon'); if (a) a.innerHTML = icon(i, 4);
    },
    deskWs(i) { set($('dws'), 'textContent', pic(i)); const a = $('dicon'); if (a) a.innerHTML = icon(i); },
    deskWx(k, c) { set($('dwx'), 'textContent', `${SKY[k] ?? SKY.clear} ${c}°`); },
  },
};
