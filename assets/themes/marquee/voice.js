// Marquee — a crime picture in five acts: acts for rooms, poster tag lines for verbs, the opening titles for a bar.
import { $, set, store, esc } from './engine.js';
import { WS } from './gen.js';

const MON = ['JAN', 'FEB', 'MAR', 'APR', 'MAY', 'JUN', 'JUL', 'AUG', 'SEP', 'OCT', 'NOV', 'DEC'];
const MONTH = ['JANUARY', 'FEBRUARY', 'MARCH', 'APRIL', 'MAY', 'JUNE', 'JULY', 'AUGUST', 'SEPTEMBER', 'OCTOBER', 'NOVEMBER', 'DECEMBER'];
const DAYS = ['SUNDAY', 'MONDAY', 'TUESDAY', 'WEDNESDAY', 'THURSDAY', 'FRIDAY', 'SATURDAY'];
const DAY = ['SUN', 'MON', 'TUE', 'WED', 'THU', 'FRI', 'SAT'];
const pad = n => String(n).padStart(2, '0');
const TITLES = ['The City', 'The Stranger', 'The Road', 'The Fall', 'The End'];
const act = i => String(WS[i] ?? `Act ${i + 1}`).toUpperCase();
const title = i => TITLES[i % TITLES.length].toUpperCase();
// the weather, as the late edition has it
const SKY = { clear: 'FAIR', cloudy: 'OVERCAST', light_rain: 'DRIZZLE', heavy_rain: 'DOWNPOUR', snow: 'SNOW', thunder: 'THUNDER' };
const TAGLINES = ['Every city has a secret.', 'Nobody leaves before the last reel.', 'He came in with the rain.', 'Trust no one. Not even the lamp.',
  'Some doors only open once.', 'The truth runs ninety minutes.', 'It was never about the money.', 'Tonight, everybody pays.'];

// a countdown leader: the ring, the cross, and a wedge swept round to the value, the way the numbers count in a reel
function leader(svg, v, big) {
  if (!svg) return;
  const key = String(Math.round(v / 2));
  if (svg.dataset.v === key) return;
  svg.dataset.v = key;
  const vb = svg.viewBox.baseVal, c = vb.width / 2, r = c - 1.2, a = Math.max(0, Math.min(100, v)) / 100 * Math.PI * 2;
  const x = c + Math.sin(a) * r, y = c - Math.cos(a) * r;
  const wedge = v >= 99.5 ? `<circle class="wedge" cx="${c}" cy="${c}" r="${r}"/>`
    : v > 0.5 ? `<path class="wedge" d="M${c} ${c}V${c - r}A${r} ${r} 0 ${a > Math.PI ? 1 : 0} 1 ${x.toFixed(2)} ${y.toFixed(2)}z"/>` : '';
  svg.innerHTML = wedge + `<path class="x" d="M${c} 0V${vb.height}M0 ${c}H${vb.width}"/><circle class="o" cx="${c}" cy="${c}" r="${r}"/>` +
    (big ? `<circle class="i" cx="${c}" cy="${c}" r="${(r * 0.8).toFixed(2)}"/>` : '');
}

// the showtime board: each letter a plastic tile pushed onto the track by hand, so none sits quite straight
function board(el, text) {
  if (!el || el.dataset.t === text) return;
  el.dataset.t = text;
  el.innerHTML = [...text].map((ch, k) => {
    if (ch === ' ') return '<i class="sp"></i>';
    const h = ((k + 1) * 7919 + ch.charCodeAt(0) * 104729) % 1000 / 1000;
    const rot = ((h * 13) % 1 - 0.5) * 3.2, dy = ((h * 29) % 1 - 0.5) * 2.4;
    return `<b style="--r:${rot.toFixed(2)}deg;--y:${dy.toFixed(2)}px">${esc(ch)}</b>`;
  }).join('');
}

export default {
  // each act's title (the dash's poster, the tooltip, the middle of the bar when nothing is open)
  ws: TITLES,
  day: DAY,
  dayLong: DAYS.map(s => s[0] + s.slice(1).toLowerCase()),
  clock: d => ({ hm: d.toTimeString().slice(0, 5), day: `${DAY[d.getDay()]} ${pad(d.getDate())} ${MON[d.getMonth()]}` }),
  wx: SKY,
  // the cast: what part each window plays
  app: { windowsterminal: 'THE MACHINE', zen: 'THE WITNESS', firefox: 'THE WITNESS', chrome: 'THE WITNESS', msedge: 'THE WITNESS', brave: 'THE WITNESS',
    opera: 'THE WITNESS', code: 'THE FORGER', cursor: 'THE FORGER', explorer: 'THE FILES', files: 'THE FILES', spotify: 'THE SCORE',
    discord: 'THE INFORMANT', telegram: 'THE INFORMANT', claude: 'THE ACCOMPLICE', steam: 'THE GAMBLER', notion: 'THE LEDGER', obsidian: 'THE LEDGER',
    settings: 'THE FIXER', photoshop: 'THE FORGER', blender: 'THE SCULPTOR', vlc: 'THE CAMERA', obs64: 'THE CAMERA' },
  appWindow: 'STARRING', appIdle: 'INTERMISSION', idle: 'marquee',
  names: { windowsterminal: 'Terminal', msedge: 'Edge', code: 'VS Code', explorer: 'Files', claude: 'Claude', zen: 'Zen',
    chrome: 'Chrome', firefox: 'Firefox', brave: 'Brave', spotify: 'Spotify', discord: 'Discord', steam: 'Steam', obs64: 'OBS' },
  nowm: ['PAUSED', 'the projector is off'],
  t: { cpu: 'Projector', ram: 'Reel', battery: 'Lamp' },

  dash: {
    size: { dash: [880, 540], power: [420, 300], wall: [930, 240], keys: [900, 430] },
    date: d => { const i = +store('rs.ws') || 0; return [d.toTimeString().slice(0, 5), `${DAYS[d.getDay()]} ${d.getDate()} ${MONTH[d.getMonth()]}`, `${act(i)} — ${title(i)}`]; },
    month: (y, m) => `${MONTH[m]} ${y}`, wd: ['S', 'M', 'T', 'W', 'T', 'F', 'S'],
    rings: [['cpu', 'PROJECTOR', 'running'], ['ram', 'REEL', 'loaded'], ['disk', 'VAULT', 'filled'], ['bat', 'LAMP', 'burning']],
    idleMedia: ['NO SCORE', 'the orchestra is out'],
    mottos: TAGLINES,
    power: { lock: ['FADE OUT', 'Lock'], sleep: ['INTERMISSION', 'Sleep'], restart: ['RE-RUN', 'Restart'], off: ['THE END', 'Shut down'],
      idle: ['LAST REEL', 'How does it end?', 'Esc to keep watching'], confirm: 'cut again to confirm', twice: 'cut twice' },
    walls: [['ACT I', 'The City'], ['ACT II', 'The Stranger'], ['ACT III', 'The Road'], ['ACT IV', 'The Fall'], ['FINALE', 'The End'], ['B-SIDE', 'Yours']],
    wallOn: 'NOW SHOWING', wallBusy: 'THREADING THE REEL …', wallFail: 'the film snapped',
    keys: {
      windows: ['SCENE', 'Windows', 'Ctrl + Alt +'], workspaces: ['ACTS', 'The acts', 'Ctrl + Alt +'], escape: ['CUT!', 'When it goes wrong'],
      bar: ['MARQUEE', 'The bar', 'mouse'],
      barRows: [['clapper', 'overview'], ['acts', 'go · wheel · right: send'], ['tag lines', 'Find it · Files · Machine · Script'],
        ['starring', 'wheel focus · 2× maximize'], ['', 'right: float · middle: close'], ['‹ tray', 'tray icons drawer'],
        ['score', 'play · right: next · mid: prev'], ['leaders', 'Task Manager'], ['lamp', 'battery settings'],
        ['volume', 'wheel · click mute · right: sound'], ['running time', 'dashboard · right: notifications'], ['end card', 'lock · sleep · restart · off']],
    },
  },

  // the desk widget is the cinema's showtime board; each new minute the bulbs chase once round it
  desk(d) {
    board($('dday'), `${DAY[d.getDay()]} ${d.getDate()} ${MON[d.getMonth()]}`);
    board($('dtime'), d.toTimeString().slice(0, 5));
    const b = $('board');
    if (b) { b.classList.remove('chase'); void b.offsetWidth; b.classList.add('chase'); }
    if (!$('dact')?.dataset.t) this.fx.deskWs(+store('rs.ws') || 0);
  },

  fx: {
    // an act: its card, cut in on white when it is on screen (bar.css)
    wsButton: n => `<span class="wn">${esc(n)}</span>`,
    ring: (svg, v) => leader(svg, v, false),
    dashRingHTML: (id, k, en) => `<div class="ring2" id="g-${id}"><div class="lead"><svg viewBox="0 0 64 64" id="r-${id}"></svg><em id="v-${id}">–</em></div>` +
      `<div class="rt"><b>${k}</b><span id="s-${id}">${en}</span></div></div>`,
    dashRing(id, v, reset) { const s = $(`r-${id}`); if (reset && s) s.dataset.v = ''; leader(s, reset ? 0 : v, true); },
    // the one-sheet drops in and the credits roll up under it
    dashOpen() { const b = document.body; b.classList.remove('roll'); void b.offsetWidth; b.classList.add('roll'); },
    deskWs(i) { board($('dact'), `${act(i)}  ${title(i)}`); },
    deskWx(k, c) { board($('dwx'), `TONIGHT ${SKY[k] ?? SKY.clear} ${c}°`); },
  },
};
