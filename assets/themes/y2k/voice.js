// Y2K Blue — a desktop from 2001: glossy blue gel, chrome, pixel stars and one magenta highlight. Five folders (home,
// music, pics, chat, games), each a little glossy icon; every word lowercase and friendly; a pixel sparkle wherever
// something is pointed at, and things that open bounce in.
import { $, set, store, esc } from './engine.js';
import { WS } from './gen.js';

const MON = ['jan', 'feb', 'mar', 'apr', 'may', 'jun', 'jul', 'aug', 'sep', 'oct', 'nov', 'dec'];
const MONTHS = ['january', 'february', 'march', 'april', 'may', 'june', 'july', 'august', 'september', 'october', 'november', 'december'];
const DAY = ['sun', 'mon', 'tue', 'wed', 'thu', 'fri', 'sat'];
const DAYS = ['sunday', 'monday', 'tuesday', 'wednesday', 'thursday', 'friday', 'saturday'];
const SHOWS = ['grid horizon', 'cd rainbow', 'pixel collage', 'liquid chrome', 'press start'];
const ABOUT = ['a chrome ball floating over the grid, the sky full of stars.', 'a cd turned to the light, its rainbow running round.',
  'stickers, a little window and a welcome banner.', 'three drops of liquid chrome pulling apart.', 'a ringed planet, a little ship, one more go.'];
const SKY = { clear: 'sunny', cloudy: 'cloudy', light_rain: 'drizzly', heavy_rain: 'rainy', snow: 'snowy', thunder: 'stormy' };
const folder = i => String(WS[i] ?? `folder ${i + 1}`).toLowerCase();
// the folder icons, drawn on a 16-pixel grid: a house, a note, a picture, a speech bubble, a game pad
const ICON = [
  'M3 8l5-5 5 5v6H3zM7 11v3h2v-3z', 'M6 3h7v8.5a2 2 0 1 1-1.5-1.9V5H7.5v7.5A2 2 0 1 1 6 10.6z',
  'M2 3h12v10H2zM3.5 11.5l3-4 2 2.5 1.5-1.5 2.5 3zM10.5 5.5a1 1 0 1 0 0 2 1 1 0 0 0 0-2z', 'M2 3h12v8H8l-3 3v-3H2z',
  'M1 6h14v6H1zM3.5 8.5h1v-1h1v1h1v1h-1v1h-1v-1h-1zM10.5 8h1.2v1.2h-1.2zM12.3 9.4h1.2v1.2h-1.2z'];
const icon = i => `<svg class="fi" viewBox="0 0 16 16"><path fill-rule="evenodd" d="${ICON[i % 5]}"/></svg>`;

// a figure that changed pops once (bar.css .pop: up and back with a little overshoot)
function pop(el, s) {
  if (!el || el.textContent === s) return;
  el.textContent = s;
  el.classList.remove('pop'); void el.offsetWidth; el.classList.add('pop');
}

export default {
  ws: SHOWS,
  day: DAY,
  dayLong: DAYS,
  clock: d => ({ hm: d.toTimeString().slice(0, 5), day: `${DAY[d.getDay()]} ${d.getDate()} ${MON[d.getMonth()]}` }),
  wx: SKY,
  // each program, the way a 2001 desktop would have called it
  app: { windowsterminal: 'dos box', zen: 'the web', firefox: 'the web', chrome: 'the web', msedge: 'the web', brave: 'the web', opera: 'the web',
    code: 'code', cursor: 'code', explorer: 'my files', files: 'my files', spotify: 'music', discord: 'chat', telegram: 'chat', claude: 'assistant',
    steam: 'games', notion: 'notes', obsidian: 'notes', settings: 'control panel', photoshop: 'paint', blender: '3d', vlc: 'video', obs64: 'recorder' },
  appWindow: 'window', appIdle: 'desktop', idle: 'nothing open',
  names: { windowsterminal: 'Terminal', msedge: 'Edge', code: 'VS Code', explorer: 'Files', claude: 'Claude', zen: 'Zen',
    chrome: 'Chrome', firefox: 'Firefox', brave: 'Brave', spotify: 'Spotify', discord: 'Discord', steam: 'Steam', obs64: 'OBS' },
  nowm: ['no folder', 'nothing in this folder yet'],
  t: { cpu: 'cpu', ram: 'mem', battery: 'batt' },

  dash: {
    size: { dash: [900, 540], power: [460, 320], wall: [940, 260], keys: [900, 440] },
    closeMs: 200,
    date: d => [d.toTimeString().slice(0, 5), `${DAYS[d.getDay()]} ${d.getDate()} ${MONTHS[d.getMonth()]}`,
      `${d.getFullYear()} · day ${Math.floor((d - new Date(d.getFullYear(), 0, 1)) / 864e5) + 1}`],
    month: (y, m) => `${MONTHS[m]} ${y}`, wd: ['s', 'm', 't', 'w', 't', 'f', 's'],
    rings: [['cpu', 'cpu', 'busy'], ['ram', 'mem', 'used'], ['disk', 'disk', 'full'], ['bat', 'batt', 'charge']],
    idleMedia: ['nothing playing', 'put a song on :)'],
    mottos: ['have a nice day :)', "you've got mail!", 'brb, saving to floppy', 'welcome to my homepage', 'press start to continue',
      'best viewed at 1024×768', 'loading… please wait', 'sign my guestbook!'],
    power: { lock: ['lock', 'lock the screen'], sleep: ['sleep', 'nap time'], restart: ['restart', 'turn it off and on again'],
      off: ['shut down', 'see you later!'], idle: ['bye for now?', 'pick one', 'esc to cancel'], confirm: 'click again to be sure', twice: 'click twice' },
    walls: [['home', 'grid horizon'], ['music', 'cd rainbow'], ['pics', 'pixel collage'], ['chat', 'liquid chrome'], ['games', 'press start'], ['yours', 'your own']],
    wallOn: 'on', wallBusy: 'loading…', wallFail: 'oops',
    keys: {
      windows: ['1', 'windows', 'Ctrl + Alt +'], workspaces: ['2', 'folders', 'Ctrl + Alt +'], escape: ['3', 'get out'],
      bar: ['4', 'the bar', 'mouse'],
      barRows: [['star', 'overview'], ['home … games', 'go · wheel · right: send'], ['search … keys', 'Flow · files · dos · this'],
        ['window', 'wheel focus · 2× maximize'], ['', 'right: float · mid: close'], ['‹ tray', 'tray icons'],
        ['music', 'play · right: next · mid: prev'], ['cpu · mem', 'Task Manager'], ['batt', 'battery settings'],
        ['volume', 'wheel · click: mute'], ['clock', 'dashboard · right: alerts'], ['power', 'lock · sleep · restart · off']],
    },
  },

  // the desk buddy: a little window with the time in chrome and a blinking cursor after it; the day, its folder, the weather
  desk(d) {
    set($('dhm'), 'textContent', d.toTimeString().slice(0, 5));
    set($('dday'), 'textContent', `${DAYS[d.getDay()]} ${d.getDate()} ${MONTHS[d.getMonth()]}`);
    if (!$('dws')?.innerHTML) this.fx.deskWs(+store('rs.ws') || 0);
  },

  fx: {
    // a folder: its glossy icon and its name
    wsButton: (n, i) => `${icon(i)}<span class="fn">${esc(String(n).toLowerCase())}</span>`,
    ring: (el, v) => pop(el, String(Math.round(Math.max(0, Math.min(99, v))))),
    // the folder you go to hops once (bar.css .hop: the springy bounce, 360 ms)
    onActive: (b, i, last, reduced) => { if (reduced || i === last || !b[i]) return; b[i].classList.remove('hop'); void b[i].offsetWidth; b[i].classList.add('hop'); },
    // the meters: a gel capsule filling with blue, the figure on it
    dashRingHTML: (id, k, en) => `<div class="ring2" id="g-${id}"><b>${k}</b><div class="cap2"><i id="r-${id}"></i><em id="v-${id}">–</em></div><span id="s-${id}">${en}</span></div>`,
    dashRing(id, v, reset) { $(`r-${id}`)?.style.setProperty('--p', (reset ? 0 : Math.max(0, Math.min(100, v))) / 100); },
    dashOpen(i) {
      set($('artImg'), 'src', new URL(`./art/wall-${i % 5}.jpg`, location.href).href);
      set($('about'), 'textContent', ABOUT[i % 5]);
      set($('wtitle'), 'textContent', `${folder(i)} — ${SHOWS[i % 5].replace(' ', '_')}.bmp`);
      const a = $('aicon'); if (a) a.innerHTML = icon(i);
    },
    deskWs(i) { set($('dws'), 'innerHTML', `${icon(i)}<span>${esc(folder(i))}</span>`); },
    deskWx(k, c) { set($('dwx'), 'textContent', `${SKY[k] ?? SKY.clear} · ${c}°`); },
  },
};
