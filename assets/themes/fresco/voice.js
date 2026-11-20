// Fresco: a ceiling is painted one day's patch of wet lime plaster at a time (a giornata), and the colour only holds
// while the plaster is wet. The five workspaces are the painter's five pigments. A workspace with nothing in it is only
// sketched in sinopia; one with windows is dry pigment; the one you are in is wet. Words are plain English, sentence
// case; the pigments keep their Italian names.
import { $, set, store, esc } from './engine.js';
import { WS, ART } from './gen.js';

const MON = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
const MONTHS = ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December'];
const DAY = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
const DAYS = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];
const ROMAN = ['I', 'II', 'III', 'IV', 'V', 'VI', 'VII', 'VIII', 'IX', 'X', 'XI', 'XII', 'XIII', 'XIV', 'XV', 'XVI', 'XVII', 'XVIII', 'XIX', 'XX'];
const roman = n => ROMAN[n - 1] ?? String(n);
// what each pot is for
const POTS = ['the blue of the sky', 'green earth, laid under flesh', 'the red of the first drawing', 'lead-tin yellow, for gold and light', 'rose lake, for cheeks and drapery'];
const SKY = { clear: 'Clear', cloudy: 'Cloudy', light_rain: 'Drizzle', heavy_rain: 'Rain', snow: 'Snow', thunder: 'Storm' };
const pig = i => String(WS[i] ?? roman(i + 1));
const DAYMS = 864e5;

// the wet pigment everywhere at once (bar, panel, desk): giallorino is pale, so what sits on it is dark
function wet(i) {
  const r = document.documentElement.style, k = i % 5;
  r.setProperty('--pig', `var(--p${k})`); r.setProperty('--on-pig', k === 3 ? 'var(--ink)' : 'var(--plaster)');
}
// a word that changed settles out of a blur (bar.css .ink)
function settle(el) { if (!el) return; el.classList.remove('ink'); void el.offsetWidth; el.classList.add('ink'); }

export default {
  ws: POTS,
  day: DAY,
  dayLong: DAYS,
  clock: d => ({ hm: d.toTimeString().slice(0, 5), day: `${DAY[d.getDay()]} ${d.getDate()} ${MON[d.getMonth()]}` }),
  wx: SKY,
  // what kind of thing is open, in the painter's words where they fit and plain ones where they don't
  app: { windowsterminal: 'Terminal', zen: 'Browser', firefox: 'Browser', chrome: 'Browser', msedge: 'Browser', brave: 'Browser', opera: 'Browser',
    code: 'Code', cursor: 'Code', explorer: 'Files', files: 'Files', spotify: 'Music', discord: 'Talk', telegram: 'Talk', claude: 'Assistant',
    steam: 'Games', notion: 'Notes', obsidian: 'Notes', settings: 'Settings', photoshop: 'Painting', blender: 'Modelling', vlc: 'Film', obs64: 'Recording' },
  appWindow: 'Window', appIdle: 'Bare wall', idle: 'Nothing open',
  names: { windowsterminal: 'Terminal', msedge: 'Edge', code: 'VS Code', explorer: 'Files', claude: 'Claude', zen: 'Zen',
    chrome: 'Chrome', firefox: 'Firefox', brave: 'Brave', spotify: 'Spotify', discord: 'Discord', steam: 'Steam', obs64: 'OBS' },
  nowm: ['Scaffold down', 'Windows are not being tiled'],
  t: { cpu: 'cpu', ram: 'memory', battery: 'battery' },

  dash: {
    size: { dash: [920, 560], power: [480, 300], wall: [980, 500], keys: [920, 470] },
    closeMs: 260,
    date: d => [d.toTimeString().slice(0, 5), `${DAYS[d.getDay()]} ${d.getDate()} ${MONTHS[d.getMonth()]}`,
      `${d.getFullYear()}, day ${Math.floor((d - new Date(d.getFullYear(), 0, 1)) / DAYMS) + 1}`],
    month: (y, m) => `${MONTHS[m]} ${y}`, wd: ['S', 'M', 'T', 'W', 'T', 'F', 'S'],
    rings: [['cpu', 'cpu', 'busy'], ['ram', 'memory', 'in use'], ['disk', 'disk', 'full'], ['bat', 'battery', 'charge']],
    idleMedia: ['Nothing playing', 'Put something on while you paint'],
    mottos: ['Paint it while the plaster is wet.', 'One patch a day, and the ceiling gets done.', 'What you paint today dries for good.',
      'Draw it in sinopia first.', 'Pounce the cartoon, then follow the dots.', 'Lime holds the colour; let it.', 'Mix only what today needs.',
      'Lay no more plaster than you can paint by dusk.'],
    power: { lock: ['Lock', 'Lock the screen'], sleep: ['Sleep', 'Let the plaster rest'], restart: ['Restart', 'Start a fresh patch'],
      off: ['Shut down', 'Down off the scaffold'], idle: ['Done for the day?', 'Choose one', 'Esc to stay'], confirm: 'Click again to be sure', twice: 'Needs two clicks' },
    walls: [['I', 'Azzurro'], ['II', 'Verdaccio'], ['III', 'Sinopia'], ['IV', 'Giallorino'], ['V', 'Lacca'], ['Yours', 'From your pictures']],
    wallKey: i => roman(i + 1),
    wallOn: 'On the wall', wallBusy: 'Plastering…', wallFail: "That didn't take",
    keys: {
      windows: ['I', 'Windows', 'Ctrl + Alt +'], workspaces: ['II', 'Pigments', 'Ctrl + Alt +'], escape: ['III', 'Getting out'],
      bar: ['IV', 'The bar', 'mouse'],
      barRows: [['Ring', 'overview'], ['I to V', 'go; wheel; right-click: send'], ['Search, Files', 'Flow; File Explorer'],
        ['Terminal, Keys', 'Terminal; this sheet'], ['Window name', 'wheel: focus; double: maximize'], ['', 'right: float; middle: close'],
        ['Arrow', 'tray icons'], ['Song', 'play; right: next; middle: previous'], ['cpu, memory', 'Task Manager'],
        ['Volume', 'wheel; click: mute'], ['Hour', 'this panel; right: notifications'], ['Power', 'lock, sleep, restart, shut down']],
    },
  },

  // the desk: a fresh patch of plaster on the wall, the hour cut into it in the Roman face, the wet pigment under it
  desk(d) {
    const hm = d.toTimeString().slice(0, 5);
    if ($('dhm')?.textContent !== hm) { set($('dhm'), 'textContent', hm); settle($('dhm')); }
    set($('dday'), 'textContent', `${DAYS[d.getDay()]} ${d.getDate()} ${MONTHS[d.getMonth()]}`);
    // how much of the working light is left: real hours until 20:00, the hour a painter puts the brushes down
    const left = Math.max(0, 20 * 60 - (d.getHours() * 60 + d.getMinutes()));
    set($('dleft'), 'textContent', left ? `${Math.floor(left / 60)} h ${left % 60} min of light left` : 'Brushes down');
    if (!$('dpig')?.textContent) this.fx.deskWs(+store('rs.ws') || 0);
  },

  fx: {
    wsButton: (n, i) => `<span>${roman(i + 1)}</span>`,
    barReady() {
      document.querySelectorAll('#ws .w').forEach((b, i) => {
        b.classList.add(`p${i % 5}`);
        b.title = `${pig(i)}, ${POTS[i % 5]}. Ctrl+Alt+${i + 1}; right-click: send the window here`;
      });
      // a new window's name and a new minute settle out of a blur
      for (const id of ['tapp', 'hm']) {
        const el = $(id); if (!el) continue;
        new MutationObserver(() => settle(el)).observe(el, { childList: true, characterData: true, subtree: true });
      }
    },
    ring(el, v) {
      set(el, 'textContent', String(Math.round(Math.max(0, Math.min(99, v)))));
      el?.classList.toggle('hot', v >= 85);
    },
    // the pigment you go to soaks into its tile; the trowel sheen crosses it once; its name comes up beside the tiles
    onActive(b, i, last, quiet) {
      wet(i);
      set($('pigName'), 'textContent', pig(i));
      if (quiet || !b[i]) return;
      b[i].classList.remove('trowel'); void b[i].offsetWidth; b[i].classList.add('trowel');
      const p = $('pig'); p.classList.remove('new'); void p.offsetWidth; p.classList.add('new');
    },

    // the meters: a stroke of the wet pigment laid across a dotted guide, longer as the figure grows
    dashRingHTML: (id, k, en) => `<div class="ring2" id="g-${id}"><b>${esc(k)}</b><em id="v-${id}">-</em><span id="s-${id}">${esc(en)}</span><i class="guide"><i id="r-${id}"></i></i></div>`,
    dashRing(id, v, reset) { $(`r-${id}`)?.style.setProperty('--p', (reset ? 0 : Math.max(0, Math.min(100, v))) / 100); },
    dashOpen(i) {
      const k = i % 5;
      wet(i);
      if (i < ART.ws) set($('artImg'), 'src', new URL(`./art/ws-${i}.jpg`, location.href).href);
      set($('artEn'), 'textContent', POTS[k][0].toUpperCase() + POTS[k].slice(1));
      set($('artK'), 'textContent', pig(i));
      set($('artN'), 'textContent', roman(i + 1));
      const a = $('lunette'); if (a) { a.classList.remove('wet'); void a.offsetWidth; a.classList.add('wet'); }
    },
    deskWs(i) {
      wet(i);
      set($('dpig'), 'textContent', pig(i));
    },
    deskWx(k, c) { set($('dwx'), 'textContent', `${SKY[k] ?? SKY.clear}, ${c}°`); },
  },
};
