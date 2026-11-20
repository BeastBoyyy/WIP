// Relic: a vanitas shelf in a dark room with one candle on it, and the candle is the day. Its wax is the part of the day
// still left, so it is tall in the morning and a stub by night. The five workspaces are the objects of a vanitas
// painting (Skull, Crown, Key, Book, Butterfly), each with what it stands for. The sayings are the old Latin ones, with
// their English and, where there is one, who said it.
import { $, set, store, esc } from './engine.js';
import { WS, ART } from './gen.js';

const MON = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
const MONTHS = ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December'];
const DAY = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
const DAYS = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];
const MEANS = ['Remember that you will die', 'Power passes to someone else', 'What is locked away', 'All that is learned', 'The soul, and how briefly it stays'];
const SKY = { clear: 'Clear', cloudy: 'Overcast', light_rain: 'Drizzle', heavy_rain: 'Rain', snow: 'Snow', thunder: 'Thunder' };
const name = i => String(WS[i] ?? `Object ${i + 1}`);
const DAYMS = 864e5;

// the candle: how much of the day is left, as wax (never quite gone), and in words
function wax(d) {
  const left = 1440 - (d.getHours() * 60 + d.getMinutes());
  document.documentElement.style.setProperty('--wax', Math.max(0.08, left / 1440).toFixed(3));
  return left;
}
function burning(left) {
  const h = Math.floor(left / 60), m = left % 60;
  if (h >= 1) return `${h} hour${h === 1 ? '' : 's'} of candle left`;
  return `${m} minute${m === 1 ? '' : 's'} of candle left`;
}
// a word that changed comes out of the dark (bar.css .lit)
function lit(el) { if (!el) return; el.classList.remove('lit'); void el.offsetWidth; el.classList.add('lit'); }

export default {
  ws: MEANS,
  day: DAY,
  dayLong: DAYS,
  clock: d => { wax(d); return { hm: d.toTimeString().slice(0, 5), day: `${DAY[d.getDay()]} ${d.getDate()} ${MON[d.getMonth()]}` }; },
  wx: SKY,
  app: { windowsterminal: 'Terminal', zen: 'Browser', firefox: 'Browser', chrome: 'Browser', msedge: 'Browser', brave: 'Browser', opera: 'Browser',
    code: 'Code', cursor: 'Code', explorer: 'Files', files: 'Files', spotify: 'Music', discord: 'Letters', telegram: 'Letters', claude: 'Assistant',
    steam: 'Games', notion: 'Notes', obsidian: 'Notes', settings: 'Settings', photoshop: 'Images', blender: 'Modelling', vlc: 'Film', obs64: 'Recording' },
  appWindow: 'Window', appIdle: 'On the shelf', idle: 'Nothing open',
  names: { windowsterminal: 'Terminal', msedge: 'Edge', code: 'VS Code', explorer: 'Files', claude: 'Claude', zen: 'Zen',
    chrome: 'Chrome', firefox: 'Firefox', brave: 'Brave', spotify: 'Spotify', discord: 'Discord', steam: 'Steam', obs64: 'OBS' },
  nowm: ['Candle held still', 'Windows are not being tiled'],
  t: { cpu: 'cpu', ram: 'memory', battery: 'battery' },

  dash: {
    size: { dash: [940, 560], power: [520, 300], wall: [980, 520], keys: [920, 460] },
    closeMs: 260,
    date: d => {
      set($('dleft'), 'textContent', burning(wax(d)));
      return [d.toTimeString().slice(0, 5), `${DAYS[d.getDay()]} ${d.getDate()} ${MONTHS[d.getMonth()]}`,
        `${d.getFullYear()}, day ${Math.floor((d - new Date(d.getFullYear(), 0, 1)) / DAYMS) + 1}`];
    },
    month: (y, m) => `${MONTHS[m]} ${y}`, wd: ['S', 'M', 'T', 'W', 'T', 'F', 'S'],
    rings: [['cpu', 'Processor', 'busy'], ['ram', 'Memory', 'in use'], ['disk', 'Disk', 'full'], ['bat', 'Battery', 'charge']],
    idleMedia: ['Nothing playing', 'The room is quiet'],
    mottos: ['Memento mori. Remember that you will die.', 'Vanitas vanitatum. Vanity of vanities (Ecclesiastes).',
      'Tempus fugit. Time flies (Virgil).', 'Ars longa, vita brevis. Art is long, life is short (Hippocrates).',
      'Sic transit gloria mundi. So passes the glory of the world.', 'Omnia mors aequat. Death makes all things equal (Claudian).',
      'Carpe diem. Seize the day (Horace).', 'Ultima latet. The last hour is hidden.'],
    power: { lock: ['Lock', 'Close the cabinet'], sleep: ['Sleep', 'Shade the candle'], restart: ['Restart', 'Light a new candle'],
      off: ['Shut down', 'Snuff it out'], idle: ['Snuff the candle?', 'Choose one', 'Esc to stay'], confirm: 'Click again to be sure', twice: 'Needs two clicks' },
    walls: [['S', 'Skull'], ['C', 'Crown'], ['K', 'Key'], ['B', 'Book'], ['B', 'Butterfly'], ['Yours', 'From your pictures']],
    wallKey: i => `No. ${i + 1}`,
    wallOn: 'Hanging', wallBusy: 'Hanging it…', wallFail: "It wouldn't hang",
    keys: {
      windows: ['1', 'Windows', 'Ctrl + Alt +'], workspaces: ['2', 'The shelf', 'Ctrl + Alt +'], escape: ['3', 'Leaving'],
      bar: ['4', 'The bar', 'mouse'],
      barRows: [['Candle and hour', 'this panel; right: notifications'], ['Skull to Butterfly', 'go; wheel; right-click: send'],
        ['Hourglass', 'overview'], ['Window name', 'wheel: focus; double: maximize'], ['', 'right: float; middle: close'],
        ['Search, Files', 'Flow; File Explorer'], ['Terminal, Keys', 'Terminal; this sheet'], ['Arrow', 'tray icons'],
        ['Song', 'play; right: next; middle: previous'], ['cpu, mem', 'Task Manager'], ['Volume', 'wheel; click: mute'],
        ['Power', 'lock, sleep, restart, shut down']],
    },
  },

  // the desk: the hour in italic beside a tall candle, how much candle is left, and a museum label for the object you are with
  desk(d) {
    const hm = d.toTimeString().slice(0, 5);
    if ($('dhm')?.textContent !== hm) { set($('dhm'), 'textContent', hm); lit($('dhm')); }
    set($('dleft'), 'textContent', burning(wax(d)));
    set($('dday'), 'textContent', `${DAYS[d.getDay()]} ${d.getDate()} ${MONTHS[d.getMonth()]} ${d.getFullYear()}`);
    if (!$('dobj')?.textContent) this.fx.deskWs(+store('rs.ws') || 0);
  },

  fx: {
    wsButton: (n, i) => `<span class="g">${esc(name(i))}</span>`,
    barReady() {
      document.querySelectorAll('#ws .w').forEach((b, i) => { b.title = `${name(i)}: ${MEANS[i % 5].toLowerCase()}. Ctrl+Alt+${i + 1}; right-click: send the window here`; });
      for (const id of ['tapp', 'hm']) {
        const el = $(id); if (!el) continue;
        new MutationObserver(() => lit(el)).observe(el, { childList: true, characterData: true, subtree: true });
      }
    },
    ring(el, v) {
      set(el, 'textContent', String(Math.round(Math.max(0, Math.min(99, v)))));
      el?.classList.toggle('hot', v >= 85);
    },
    onActive(b, i, last, quiet) { if (!quiet && b[i]) lit(b[i]); },

    // the meters: a fine gilt rule drawn out along a hairline, a small lozenge where it stops
    dashRingHTML: (id, k, en) => `<div class="ring2" id="g-${id}"><b>${esc(k)}</b><i class="rule"><i id="r-${id}"></i></i><em id="v-${id}">-</em><span id="s-${id}">${esc(en)}</span></div>`,
    dashRing(id, v, reset) { $(`r-${id}`)?.style.setProperty('--p', (reset ? 0 : Math.max(0, Math.min(100, v))) / 100); },
    dashOpen(i) {
      if (i < ART.ws) set($('artImg'), 'src', new URL(`./art/ws-${i}.jpg`, location.href).href);
      set($('artK'), 'textContent', name(i));
      set($('artEn'), 'textContent', MEANS[i % 5]);
      set($('artN'), 'textContent', `No. ${i + 1} of ${WS.length}`);
      const a = $('art'); if (a) { a.classList.remove('wake'); void a.offsetWidth; a.classList.add('wake'); }
    },
    deskWs(i) { set($('dobj'), 'textContent', name(i)); set($('dmeans'), 'textContent', MEANS[i % 5]); },
    deskWx(k, c) { set($('dwx'), 'textContent', `${SKY[k] ?? SKY.clear}, ${c}°`); },
  },
};
