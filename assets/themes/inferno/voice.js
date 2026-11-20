// Inferno: everything is a black shape against fire, lit from below. The five workspaces are the five rivers of the
// underworld (woe, hate, fire, wailing, forgetting), shown by their blackletter initials. Plain English words; the
// sayings on the dashboard are Dante (in Longfellow's translation) and Milton, credited.
import { $, set, store, esc } from './engine.js';
import { WS, ART } from './gen.js';

const MON = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
const MONTHS = ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December'];
const DAY = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
const DAYS = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];
const OF = ['woe', 'hate', 'fire', 'wailing', 'forgetting'];
const SKY = { clear: 'Clear', cloudy: 'Smoke', light_rain: 'Drizzle', heavy_rain: 'Rain', snow: 'Snow', thunder: 'Storm' };
const river = i => String(WS[i] ?? `River ${i + 1}`);
const initial = i => river(i).trim()[0] ?? String(i + 1);
const DAYMS = 864e5;

// a word that changed comes up out of the floor (bar.css .lit)
function lit(el) { if (!el) return; el.classList.remove('lit'); void el.offsetWidth; el.classList.add('lit'); }

export default {
  ws: OF.map(o => `the river of ${o}`),
  day: DAY,
  dayLong: DAYS,
  clock: d => ({ hm: d.toTimeString().slice(0, 5), day: `${DAY[d.getDay()]} ${d.getDate()} ${MON[d.getMonth()]}` }),
  wx: SKY,
  app: { windowsterminal: 'Terminal', zen: 'Browser', firefox: 'Browser', chrome: 'Browser', msedge: 'Browser', brave: 'Browser', opera: 'Browser',
    code: 'Code', cursor: 'Code', explorer: 'Files', files: 'Files', spotify: 'Music', discord: 'Talk', telegram: 'Talk', claude: 'Assistant',
    steam: 'Games', notion: 'Notes', obsidian: 'Notes', settings: 'Settings', photoshop: 'Images', blender: 'Modelling', vlc: 'Film', obs64: 'Recording' },
  appWindow: 'Window', appIdle: 'The dark', idle: 'Nothing open',
  names: { windowsterminal: 'Terminal', msedge: 'Edge', code: 'VS Code', explorer: 'Files', claude: 'Claude', zen: 'Zen',
    chrome: 'Chrome', firefox: 'Firefox', brave: 'Brave', spotify: 'Spotify', discord: 'Discord', steam: 'Steam', obs64: 'OBS' },
  nowm: ['Fire banked', 'Windows are not being tiled'],
  t: { cpu: 'cpu', ram: 'memory', battery: 'battery' },

  dash: {
    size: { dash: [940, 560], power: [500, 300], wall: [980, 500], keys: [920, 400] },
    closeMs: 240,
    date: d => [d.toTimeString().slice(0, 5), `${DAYS[d.getDay()]} ${d.getDate()} ${MONTHS[d.getMonth()]}`,
      `${d.getFullYear()}, day ${Math.floor((d - new Date(d.getFullYear(), 0, 1)) / DAYMS) + 1}`],
    month: (y, m) => `${MONTHS[m]} ${y}`, wd: ['S', 'M', 'T', 'W', 'T', 'F', 'S'],
    rings: [['cpu', 'cpu', 'busy'], ['ram', 'memory', 'in use'], ['disk', 'disk', 'full'], ['bat', 'battery', 'charge']],
    idleMedia: ['Nothing playing', 'The fire is quiet'],
    mottos: ['Midway upon the journey of our life. (Dante)', 'All hope abandon, ye who enter in! (Dante)', 'The mind is its own place. (Milton)',
      'Better to reign in Hell, than serve in Heaven. (Milton)', 'Long is the way and hard, that out of Hell leads up to light. (Milton)',
      'There is no greater sorrow than to be mindful of the happy time in misery. (Dante)', 'Thence we came forth to rebehold the stars. (Dante)'],
    power: { lock: ['Lock', 'Bar the gate'], sleep: ['Sleep', 'Bank the fire'], restart: ['Restart', 'Rise again'],
      off: ['Shut down', 'Put the fire out'], idle: ['Leave the fire?', 'Choose one', 'Esc to stay'], confirm: 'Click again to be sure', twice: 'Needs two clicks' },
    walls: [['A', 'Acheron'], ['S', 'Styx'], ['P', 'Phlegethon'], ['C', 'Cocytus'], ['L', 'Lethe'], ['Yours', 'From your pictures']],
    wallKey: i => String(i + 1),
    wallOn: 'Burning', wallBusy: 'Lighting…', wallFail: "It wouldn't catch",
    keys: {
      windows: ['I', 'Windows', 'Ctrl + Alt +'], workspaces: ['II', 'Rivers', 'Ctrl + Alt +'], escape: ['III', 'The way out'],
      bar: ['IV', 'The bar', 'mouse'],
      barRows: [['Gate', 'overview'], ['A S P C L', 'go; wheel; right-click: send'], ['Search, Files', 'Flow; File Explorer'],
        ['Terminal, Keys', 'Terminal; this sheet'], ['Window name', 'wheel: focus; double: maximize'], ['', 'right: float; middle: close'],
        ['Arrow', 'tray icons'], ['Song', 'play; right: next; middle: previous'], ['cpu, memory', 'Task Manager'],
        ['Volume', 'wheel; click: mute'], ['Hour', 'this panel; right: notifications'], ['Power', 'lock, sleep, restart, shut down']],
    },
  },

  // the desk: the hour burning in blackletter on the wallpaper, the river under it, the day, the weather
  desk(d) {
    const hm = d.toTimeString().slice(0, 5);
    if ($('dhm')?.textContent !== hm) { set($('dhm'), 'textContent', hm); lit($('dhm')); }
    set($('dday'), 'textContent', `${DAYS[d.getDay()]} ${d.getDate()} ${MONTHS[d.getMonth()]}`);
    if (!$('driver')?.textContent) this.fx.deskWs(+store('rs.ws') || 0);
  },

  fx: {
    wsButton: (n, i) => `<span class="g">${esc(initial(i))}</span><i class="sp" aria-hidden="true"></i>`,
    barReady() {
      document.querySelectorAll('#ws .w').forEach((b, i) => { b.title = `${river(i)}, the river of ${OF[i % 5]}. Ctrl+Alt+${i + 1}; right-click: send the window here`; });
      for (const id of ['tapp', 'hm']) {
        const el = $(id); if (!el) continue;
        new MutationObserver(() => lit(el)).observe(el, { childList: true, characterData: true, subtree: true });
      }
    },
    ring(el, v) {
      set(el, 'textContent', String(Math.round(Math.max(0, Math.min(99, v)))));
      el?.classList.toggle('hot', v >= 85);
    },
    onActive(b, i, last, quiet) {
      set($('riverName'), 'textContent', river(i));
      set($('riverOf'), 'textContent', `of ${OF[i % 5]}`);
      if (quiet || !b[i]) return;
      b[i].classList.remove('flare'); void b[i].offsetWidth; b[i].classList.add('flare');
      const r = $('river'); r.classList.remove('new'); void r.offsetWidth; r.classList.add('new');
    },

    // the meters: columns of heat, filled from the floor up
    dashRingHTML: (id, k, en) => `<div class="ring2" id="g-${id}"><em id="v-${id}">-</em><i class="col"><i id="r-${id}"></i></i><b>${esc(k)}</b><span id="s-${id}">${esc(en)}</span></div>`,
    dashRing(id, v, reset) { $(`r-${id}`)?.style.setProperty('--p', (reset ? 0 : Math.max(0, Math.min(100, v))) / 100); },
    dashOpen(i) {
      const k = i % 5;
      if (i < ART.ws) set($('artImg'), 'src', new URL(`./art/ws-${i}.jpg`, location.href).href);
      set($('artK'), 'textContent', river(i));
      set($('artEn'), 'textContent', `The river of ${OF[k]}`);
      set($('artN'), 'textContent', initial(i));
      const a = $('art'); if (a) { a.classList.remove('burn'); void a.offsetWidth; a.classList.add('burn'); }
    },
    deskWs(i) { set($('driver'), 'textContent', river(i)); set($('dof'), 'textContent', `the river of ${OF[i % 5]}`); },
    deskWx(k, c) { set($('dwx'), 'textContent', `${SKY[k] ?? SKY.clear}, ${c}°`); },
  },
};
