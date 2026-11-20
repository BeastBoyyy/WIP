// 武蔵 Vagabond — Musashi's Book of Five Rings: sumi ink, washi paper, one red seal, and the pictures you gave it.
// Musashi's Dokkōdō lines are public domain (1645).
import { $, set, store } from './engine.js';
import { ART } from './gen.js';

const NUM = ['', '一', '二', '三', '四', '五', '六', '七', '八', '九', '十'];
const kan = n => n <= 10 ? NUM[n] : (n < 20 ? '十' : NUM[Math.floor(n / 10)] + '十') + NUM[n % 10];   // 1–31
const DAY = ['日', '月', '火', '水', '木', '金', '土'];
const PRE_JA = ['世々の道をそむく事なし', '身にたのしみをたくまず', '身を浅く思世を深く思ふ', '我事において後悔をせず',
  '常に兵法の道をはなれず', '目に見えぬ所をさとって知る', '一生の間よくしん思わず'];
const SVG = 'http://www.w3.org/2000/svg';
const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;

// a brush ring: a dry track and a wet stroke whose length is the value
function brushRing(svg, v, r, w) {
  if (!svg) return;
  const key = String(Math.round(v));
  if (svg.dataset.v === key) return;
  svg.dataset.v = key;
  const c = 2 * Math.PI * r, mid = svg.viewBox.baseVal.width / 2, arc = svg.querySelector('.arc');
  if (arc) return arc.setAttribute('stroke-dasharray', `${(c * v) / 100} ${c}`);   // same element, so the sweep transitions
  svg.innerHTML =`<circle class="track" cx="${mid}" cy="${mid}" r="${r}" stroke-width="${w}"/>` +
    `<circle class="arc" cx="${mid}" cy="${mid}" r="${r}" stroke-width="${w * 1.15}" stroke-linecap="round" filter="url(#brushy)" ` +
    `stroke-dasharray="${(c * v) / 100} ${c}" transform="rotate(-100 ${mid} ${mid})"/>`;
}

export default {
  ws: ['Earth', 'Water', 'Fire', 'Wind', 'Void'],
  day: DAY,
  dayLong: ['Sun · 日曜日', 'Mon · 月曜日', 'Tue · 火曜日', 'Wed · 水曜日', 'Thu · 木曜日', 'Fri · 金曜日', 'Sat · 土曜日'],
  wx: { clear: '晴', cloudy: '曇', light_rain: '雨', heavy_rain: '雨', snow: '雪', thunder: '雷' },
  app: { windowsterminal: '武', zen: '網', firefox: '網', chrome: '網', msedge: '網', brave: '網', opera: '網', code: '筆', cursor: '筆',
    explorer: '蔵', files: '蔵', spotify: '音', discord: '話', telegram: '話', claude: '智', steam: '遊', notion: '記', obsidian: '記',
    settings: '調', photoshop: '絵', blender: '形', vlc: '映', obs64: '映' },
  appWindow: '窓', appIdle: '武', idle: 'vagabond',
  names: { windowsterminal: 'Terminal', msedge: 'Edge', code: 'VS Code', explorer: 'Files', claude: 'Claude', zen: 'Zen',
    chrome: 'Chrome', firefox: 'Firefox', brave: 'Brave', spotify: 'Spotify', discord: 'Discord', steam: 'Steam', obs64: 'OBS' },
  nowm: ['遊', 'game mode'],
  t: { cpu: '心 CPU', ram: '記 RAM', battery: '力' },

  dash: {
    size: { dash: [800, 500], power: [320, 320], wall: [910, 350], keys: [860, 370] },
    date: d => [d.toTimeString().slice(0, 5), `${d.getMonth() + 1}月${d.getDate()}日 · ${DAY[d.getDay()]}曜日`,
      d.toLocaleDateString('en-GB', { weekday: 'long', day: 'numeric', month: 'long' })],
    month: (y, m) => `${y}年 ${m + 1}月`, wd: DAY,
    rings: [['cpu', '心', 'CPU'], ['ram', '記', 'RAM'], ['disk', '蔵', 'disk'], ['bat', '力', 'battery']],
    idleMedia: ['静', 'nothing playing'],
    mottos: ['Accept everything just the way it is.', 'Do not seek pleasure for its own sake.',
      'Think lightly of yourself and deeply of the world.', 'Do not regret what you have done.',
      'Never stray from the Way.', 'Perceive that which cannot be seen with the eye.', 'Do nothing which is of no use.'],
    power: { lock: ['鍵', 'Lock'], sleep: ['眠', 'Sleep'], restart: ['再', 'Restart'], off: ['終', 'Shut down'],
      idle: ['休', 'Rest, wanderer', 'Esc to stay'], confirm: 'click again to confirm', twice: 'asks twice' },
    walls: [['地', 'Earth'], ['水', 'Water'], ['火', 'Fire'], ['風', 'Wind'], ['空', 'Void'], ['己', 'Yours']],
    wallKey: i => kan(i + 1),
    wallOn: '今', wallBusy: '墨 …', wallFail: 'could not set it',
    keys: {
      windows: ['窓', 'Windows', 'Ctrl + Alt +'], workspaces: ['道', 'Workspaces', 'Ctrl + Alt +'], escape: ['急', 'Escape hatches'],
      bar: ['帯', 'The bar', 'mouse'],
      barRows: [['円 ensō', 'overview'], ['seals', 'go · wheel · right: send'], ['探 蔵 武', 'Flow · Files · Terminal'],
        ['title', 'wheel focus · 2× maximize'], ['', 'right: float · middle: close'], ['‹ tray', 'tray icons drawer'],
        ['song', 'play · right: next · mid: prev'], ['rings', 'Task Manager'], ['blade', 'battery settings'],
        ['volume', 'wheel · click mute · right: sound'], ['clock', 'dashboard · right: notifications'], ['⏻', 'lock · sleep · restart · off']],
    },
  },

  desk(d) {
    const hm = d.toTimeString().slice(0, 5);
    set($('dhm'), 'textContent', hm);
    const c = $('dhm'); c.classList.remove('ink'); void c.offsetWidth; c.classList.add('ink');
    set($('ddate'), 'textContent', `${kan(d.getMonth() + 1)}月${kan(d.getDate())}日 ${DAY[d.getDay()]}曜日`);
    set($('dday'), 'textContent', d.toLocaleDateString('en-GB', { weekday: 'long', day: 'numeric', month: 'long' }));
    const day = Math.floor((d - new Date(d.getFullYear(), 0, 0)) / 864e5);   // one line a day
    set($('dpre'), 'textContent', PRE_JA[day % PRE_JA.length]);
  },

  fx: {
    wsButton: name => `<span>${name}</span>`,
    // the red hanko seal stamps onto the active workspace and throws a few drops of ink
    onActive(btns, i, prev, instant) {
      const seal = $('seal'), b = btns[i];
      const x = b.offsetLeft + (b.offsetWidth - 28) / 2, rot = (Math.random() * 8 - 4).toFixed(1);
      seal.animate([
        { transform: `translateX(${x}px) scale(1.7) rotate(${rot}deg)`, opacity: 0 },
        { transform: `translateX(${x}px) scale(.88) rotate(${rot}deg)`, opacity: 1, offset: .6 },
        { transform: `translateX(${x}px) scale(1) rotate(${rot}deg)`, opacity: 1 },
      ], { duration: instant ? 1 : 260, easing: 'cubic-bezier(.22,1,.36,1)', fill: 'forwards' });
      if (instant) return;
      for (let k = 0; k < 5; k++) {
        const d = document.createElement('i'), s = 2 + Math.random() * 3, a = Math.random() * 6.28, r = 18 + Math.random() * 10;
        d.className = 'splat';
        Object.assign(d.style, { width: s + 'px', height: s + 'px', left: x + 14 + 'px', top: '16px' });
        $('ws').appendChild(d);
        d.animate([{ transform: 'translate(0,0)', opacity: .9 }, { transform: `translate(${Math.cos(a) * r}px,${Math.sin(a) * r}px)`, opacity: 0 }],
          { duration: 420, delay: 150, easing: 'ease-out' }).onfinish = () => d.remove();
      }
    },
    ring: (svg, v) => brushRing(svg, v, 15, 4),
    // the battery is a katana: the blade fills from the guard
    battery(p) { $('bfill')?.setAttribute('width', (55 * p) / 100); },
    dashRingHTML: (id, k, en) => `<div class="ring2" id="g-${id}"><div class="rw"><svg viewBox="0 0 60 60" id="r-${id}"></svg><em id="v-${id}">–</em></div><b>${k}</b><span id="s-${id}">${en}</span></div>`,
    dashRing(id, v, reset) { const s = $(`r-${id}`); if (reset && s) s.dataset.v = ''; brushRing(s, reset ? 0 : v, 24, 5); },
    // each ring of the five has its own picture on the dashboard, when you gave it one (art/ws-N.jpg)
    dashOpen(i) { if (i < ART.ws) set($('artImg'), 'src', new URL(`./art/ws-${i}.jpg`, location.href).href); },
  },
};
