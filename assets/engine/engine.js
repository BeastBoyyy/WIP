// Rice Studio bar engine: what every theme's bar, dashboard and desk widget DO. Each theme brings its own look
// (bar.html/css, dash.html/css, desk.html/css) and its own voice (voice.js: words, glyphs, motion hooks).
// Rules kept from Vagabond's ink-shell: LeopardWM pushes events (no polling except a 3 s title read while it runs),
// the DOM is only touched on change, and nothing spawns while the window manager is down (game mode).
import * as zebar from './lib/zebar.js';
import { P, WS, BAR, ART } from './gen.js';

export const $ = id => document.getElementById(id);
export const set = (el, prop, v) => { if (el && el[prop] !== v) el[prop] = v; };
export const store = (k, v) => { try { v === undefined ? (v = localStorage.getItem(k)) : localStorage.setItem(k, v); } catch { } return v; };
export const esc = t => String(t).replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const run = (prog, args = '') => zebar.shellExec(prog, args).catch(err => console.error(prog, args, err));
const launch = prog => zebar.shellSpawn(prog, '', { cwd: P.home }).catch(err => console.error(prog, err));
const ping = key => store(key, String(Date.now()));   // bar → dash: same pack, same origin, so localStorage events cross
let wheelAt = 0;   // one step per notch, even on free-spinning wheels and touchpads
export const onWheel = (el, fn) => el?.addEventListener('wheel', e => {
  e.preventDefault();
  if (Date.now() - wheelAt < 140) return;
  wheelAt = Date.now(); fn(e.deltaY > 0 ? 1 : -1);
}, { passive: false });
export const onMiddle = (el, fn) => el?.addEventListener('auxclick', e => e.button === 1 && fn());
export const onRight = (el, fn) => el?.addEventListener('contextmenu', e => { e.preventDefault(); fn(); });
const click = (id, fn) => { const el = $(id); if (el) el.onclick = fn; };
addEventListener('contextmenu', e => e.preventDefault());   // no WebView menu anywhere
addEventListener('mousedown', e => e.button === 1 && e.preventDefault());   // no autoscroll cursor on middle-click
// a hover glow that follows the pointer, for any element with class .follow (themes style --mx/--my)
document.addEventListener('pointermove', e => {
  const b = e.target.closest?.('.follow'); if (!b) return;
  const r = b.getBoundingClientRect();
  b.style.setProperty('--mx', `${e.clientX - r.left}px`); b.style.setProperty('--my', `${e.clientY - r.top}px`);
});
export const wsName = i => WS[i] ?? String(i + 1);
export const wsLabel = (V, i) => V.ws?.[i % (V.ws.length || 1)] ?? '';
const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;

/* ═══ bar ═══════════════════════════════════════════════════════════════════════════════════════════════ */
export function bar(V) {
  const fx = V.fx || {};
  // the middle island steps aside when a side one grows into it (a long song, a full tray, a narrow screen);
  // it only fades, so its box stays put and this cannot flip back and forth
  const [L, C, R] = ['iLeft', 'iCenter', 'iRight'].map($);
  if (L && C && R && window.ResizeObserver) {
    const fit = () => { const c = C.getBoundingClientRect(); document.body.classList.toggle('crowd', L.getBoundingClientRect().right + 8 > c.left || R.getBoundingClientRect().left - 8 < c.right); };
    const ro = new ResizeObserver(fit); [L, C, R].forEach(e => ro.observe(e));
  }
  const providers = zebar.createProviderGroup({
    cpu: { type: 'cpu', refreshInterval: 5000 },
    memory: { type: 'memory', refreshInterval: 10000 },
    battery: { type: 'battery', refreshInterval: 30000 },
    date: { type: 'date', formatting: 'HH:mm', refreshInterval: 1000 },   // render() only writes when the minute changes
    weather: { type: 'weather', refreshInterval: 3600000 },
    media: { type: 'media' },
    audio: { type: 'audio' },
    network: { type: 'network', refreshInterval: 30000 },
  });

  function render(o) {
    if (o.cpu && o.memory) {
      fx.ring?.($('rcpu'), o.cpu.usage, 'cpu'); fx.ring?.($('rram'), o.memory.usage, 'ram');
      set($('rings'), 'title', `${V.t.cpu} ${Math.round(o.cpu.usage)}%  ·  ${V.t.ram} ${Math.round(o.memory.usage)}%  ·  click: Task Manager`);
    }
    if (o.battery) {
      const p = Math.round(o.battery.chargePercent), chg = o.battery.isCharging;
      $('batt')?.removeAttribute('hidden');
      fx.battery ? fx.battery(p, chg) : $('bfill')?.style.setProperty('--p', p / 100);
      set($('bp'), 'textContent', `${p}%`);
      $('batt')?.classList.toggle('low', p <= 20 && !chg);
      $('batt')?.classList.toggle('chg', chg);
      set($('batt'), 'title', `${V.t.battery} ${p}%${chg ? ' · charging' : ''}  ·  click: battery settings`);
    } else $('batt')?.setAttribute('hidden', '');
    if (o.date) {
      const d = new Date(o.date.now);
      const c = V.clock ? V.clock(d) : { hm: o.date.formatted, day: V.day[d.getDay()] };
      set($('hm'), 'textContent', c.hm);
      set($('day'), 'textContent', c.day);
      set($('clock'), 'title', `${V.dayLong[d.getDay()]}  ·  click: dashboard · right-click: notifications`);
    }
    if (o.weather) {
      const base = String(o.weather.status || '').replace(/_(day|night)$/, '');
      set($('wk'), 'textContent', V.wx[base] ?? V.wx.clear);
      set($('wt'), 'textContent', `${Math.round(o.weather.celsiusTemp)}°`);
      set($('weather'), 'title', `${base.replace('_', ' ')} · ${Math.round(o.weather.celsiusTemp)}°C  ·  click: dashboard`);
      // the desk widget shows the weather too: it hears it from the bar
      const wx = `${base}|${Math.round(o.weather.celsiusTemp)}`;
      if (store('rs.wx') !== wx) store('rs.wx', wx);
    }
    const s = o.media?.currentSession;
    if ($('media')) $('media').hidden = !s;
    if (s) {
      $('media').classList.toggle('playing', !!s.isPlaying);
      set($('mt'), 'innerHTML', s.artist ? `${esc(s.artist)} · <em>${esc(s.title)}</em>` : `<em>${esc(s.title || '')}</em>`);
    }
    const dev = o.audio?.defaultPlaybackDevice;
    if (dev) {
      set($('vol'), 'textContent', `${Math.round(dev.volume)}`);
      $('volb')?.classList.toggle('muted', !!dev.isMuted);
    }
    if (o.network) {
      const up = !!o.network.defaultInterface;
      $('net')?.classList.toggle('off', !up);
      set($('net'), 'title', `${up ? (o.network.defaultGateway?.ssid || o.network.defaultInterface.type) : 'offline'}  ·  click: Wi-Fi`);
    }
  }
  render(providers.outputMap);
  providers.onOutput(() => render(providers.outputMap));

  // right side
  const media = () => providers.outputMap.media, audio = () => providers.outputMap.audio;
  click('media', () => media()?.togglePlayPause());
  onRight($('media'), () => media()?.next());
  onMiddle($('media'), () => media()?.previous());
  onWheel($('volb'), dir => {
    const a = audio(), dev = a?.defaultPlaybackDevice; if (!dev) return;
    a.setVolume(Math.max(0, Math.min(100, Math.round(dev.volume) - 2 * dir)));
  });
  click('volb', () => { const a = audio(), dev = a?.defaultPlaybackDevice; if (dev) a.setMute(!dev.isMuted); });
  onRight($('volb'), () => run(P.explorer, 'ms-settings:sound'));
  // ponytail: explorer.exe can't open the Quick Settings URI; the Wi-Fi page of Quick Settings works and ← reaches the rest
  click('net', () => run(P.explorer, 'ms-availablenetworks:'));
  click('batt', () => run(P.explorer, 'ms-settings:batterysaver'));
  click('rings', () => launch(P.taskmgr));
  click('weather', () => ping('rs.dashCmd'));
  click('clock', () => ping('rs.dashCmd'));
  onRight($('clock'), () => run(P.explorer, 'ms-actioncenter:'));
  click('pwr', () => ping('rs.powerCmd'));

  // tray drawer: the systray provider starts on first open and then stays (Zebar won't re-send icons to a new one)
  let tray = null, trayOpen = false, trayKey = '';
  function syncTray() {
    if (!trayOpen) return;
    const icons = tray?.outputMap.systray?.icons || [];
    const key = icons.map(i => i.id + i.iconHash + i.tooltip).join('|');
    if (key === trayKey) return;   // tooltips change often (volume, network): redraw only on change
    trayKey = key;
    $('tray').replaceChildren(...icons.map(ic => {
      const b = document.createElement('button');
      b.title = ic.tooltip || ''; b.innerHTML = `<img src="${ic.iconUrl}" alt="">`;
      b.onclick = () => tray?.outputMap.systray?.onLeftClick(ic.id);
      b.ondblclick = () => tray?.outputMap.systray?.onLeftDoubleClick(ic.id);
      onRight(b, () => tray?.outputMap.systray?.onRightClick(ic.id));
      onMiddle(b, () => tray?.outputMap.systray?.onMiddleClick(ic.id));
      return b;
    }));
  }
  click('trayBtn', () => {
    trayOpen = !trayOpen;
    $('trayBtn').classList.toggle('on', trayOpen);
    $('tray').hidden = !trayOpen; document.body.classList.toggle('tray', trayOpen);
    if (!trayOpen) { $('tray').replaceChildren(); trayKey = ''; return; }
    if (!tray) { tray = zebar.createProviderGroup({ systray: { type: 'systray' } }); tray.onOutput(syncTray); }
    syncTray();
  });

  // launchers
  click('lFlow', () => launch(P.flow));
  click('lFiles', () => run(P.explorer));
  click('lTerm', () => launch(P.terminal));
  click('lKeys', () => ping('rs.keysCmd'));
  click('overview', () => lwm('toggle-overview'));

  // workspaces
  const wsEl = $('ws');
  const btns = WS.map((name, i) => {
    const b = document.createElement('button');
    b.className = 'w'; b.dataset.i = i;
    b.innerHTML = fx.wsButton ? fx.wsButton(name, i) : `<span>${esc(name)}</span>`;
    b.title = `${name}${wsLabel(V, i) ? ' ' + wsLabel(V, i) : ''} · Ctrl+Alt+${i + 1} · right-click: send the focused window here`;
    b.onclick = () => lwm(`workspace ${i + 1}`);
    onRight(b, () => lwm(`move-to-workspace ${i + 1}`));
    wsEl?.appendChild(b);
    return b;
  });
  onWheel(wsEl, dir => {
    const n = Math.max(0, Math.min(WS.length - 1, active + dir));
    if (n !== active) lwm(`workspace ${n + 1}`);
  });
  let active = 0, lastActive = -1, occupied = new Set(), building = null;
  function stamp() {
    btns.forEach((b, i) => { b.classList.toggle('on', i === active); b.classList.toggle('has', i !== active && occupied.has(i)); });
    if (lastActive === active || !btns[active]) return;
    fx.onActive?.(btns, active, lastActive, lastActive < 0 || reduced);
    lastActive = active;
    store('rs.ws', active);   // the dashboard picks its art card from this
    document.body.dataset.ws = active;
  }

  // centre: the focused window
  function title(exe, t) {
    let app = (exe || '').replace(/\.exe$/i, ''), key = app.toLowerCase();
    if (key === 'applicationframehost' && t) { app = t; key = t.toLowerCase(); t = ''; }   // Store apps all run in this host
    const name = V.names[key] || app;
    const flat = s => s.toLowerCase().replace(/\s+/g, '');
    set($('tk'), 'textContent', V.app[key] ?? (app ? V.appWindow : V.appIdle));
    set($('tapp'), 'textContent', name || wsLabel(V, active) || V.idle);
    set($('tname'), 'textContent', t && ![flat(app), flat(name)].includes(flat(t)) ? ` · ${t}` : '');
  }
  onWheel($('title'), dir => lwm(`focus ${dir > 0 ? 'right' : 'left'}`));
  if ($('title')) $('title').ondblclick = () => lwm('maximize-column');
  onRight($('title'), () => lwm('toggle-floating'));
  onMiddle($('title'), () => lwm('close-window'));

  // LeopardWM sends the title only on focus change; shells and browsers rename themselves, so re-read it every 3 s
  // ponytail: `lwm query all` (~20 ms) every 3 s while LeopardWM runs; a title event would be better if it adds one
  let refreshing = false, wmUp = false;
  const lwm = args => wmUp && run(P.lwm, args);
  async function refreshTitle() {
    if (refreshing || !wmUp || document.hidden) return; refreshing = true;
    try {
      const { stdout } = await zebar.shellExec(P.lwm, 'query all');
      const m = stdout.split('\n').map(l => l.match(/^\s*\d+ - (.*) \(([^()]*)\) \[col .*\[FOCUSED\]/)).find(Boolean);
      if (m && m[2]) title(m[2], m[1]);
    } catch { /* LeopardWM just stopped */ } finally { refreshing = false; }
  }
  setInterval(refreshTitle, 3000);
  function onEvent(e) {
    switch (e.type) {
      case 'workspace_changed': active = e.new_index; stamp(); if (e.name) title(null, null); break;
      case 'focused_window_changed': title(e.executable, e.title); setTimeout(refreshTitle, 700); break;
      case 'workspace_snapshot_begin': building = new Set(); break;
      case 'workspace_snapshot_chunk':
        for (const r of e.records || []) {
          if (r.kind === 'monitor') active = r.active_workspace_index;
          if (r.kind === 'window' && building) building.add(r.workspace_index);
        }
        break;
      case 'workspace_snapshot_end': if (building) occupied = building; building = null; stamp(); break;
    }
  }
  // LeopardWM down (game mode, rice off): retry 5 s → 60 s, so a match sees about one spawn a minute
  let retry = 5000;
  const again = () => {
    wmUp = false; setTimeout(subscribe, retry); retry = Math.min(retry * 2, 60000);
    document.body.classList.add('nowm'); store('rs.nowm', '1');   // the desk stills its motion too
    set($('tk'), 'textContent', V.nowm[0]); set($('tapp'), 'textContent', V.nowm[1]); set($('tname'), 'textContent', '');
  };
  async function subscribe() {
    let buf = '';
    const p = await zebar.shellSpawn(P.lwm, 'subscribe --events workspace,focused_window,workspace_state');
    wmUp = true;   // exits at once (→ again) if LeopardWM isn't running
    p.onStdout(chunk => {
      retry = 5000;
      if (store('rs.nowm') !== '0') { document.body.classList.remove('nowm'); store('rs.nowm', '0'); }
      buf += chunk;
      let i;
      while ((i = buf.indexOf('\n')) >= 0) {
        const line = buf.slice(0, i).trim(); buf = buf.slice(i + 1);
        if (line.startsWith('{')) try { onEvent(JSON.parse(line)); } catch { /* partial line */ }
      }
    });
    p.onExit(again);
  }
  stamp();
  subscribe().catch(again);
  fx.barReady?.();
}

/* ═══ dashboard, power menu, wallpaper picker and key sheet: one pre-loaded window ═══════════════════════════ */
export function dash(V) {
  const D = V.dash, fx = V.fx || {};
  const win = zebar.currentWidget().tauriWindow;
  const PANEL = { dash: 'dash', power: 'power', wall: 'wall', keys: 'keysheet' };
  let open = false, mode = 'dash', sized = '', shownAt = 0, hiddenAt = 0, providers = null;

  async function show(m) {
    open = true; mode = m; shownAt = Date.now();
    document.body.dataset.mode = m;
    if (m === 'dash') fill(); else if (m === 'power') arm(null); else if (m === 'wall') markWall();
    const d = $(PANEL[m]); d.classList.remove('open', 'closing'); void d.offsetWidth; d.classList.add('open');
    const [w, h] = D.size[m];
    if (sized !== m) { await win.hide(); await win.setSize({ type: 'Logical', width: w, height: h }); sized = m; }
    // ponytail: assumes the primary monitor sits at (0,0), as the preset targets the primary monitor
    const p = await win.outerPosition();
    p.x = Math.round((m === 'keys' ? 0 : screen.width - w) * devicePixelRatio);
    p.y = Math.round((D.top ?? BAR + 2) * devicePixelRatio);
    await win.setPosition(p);
    await win.show();
    // the preset's top_most does not always stick (the window was seen without WS_EX_TOPMOST, under other windows);
    // tao skips a set that matches its own record, so toggle to force it
    await win.setAlwaysOnTop(false); await win.setAlwaysOnTop(true);
    if (m !== 'dash') return;
    providers = zebar.createProviderGroup({
      cpu: { type: 'cpu', refreshInterval: 2000 },
      memory: { type: 'memory', refreshInterval: 3000 },
      disk: { type: 'disk', refreshInterval: 60000 },
      battery: { type: 'battery', refreshInterval: 30000 },
      media: { type: 'media' },
    });
    requestAnimationFrame(() => { render(providers.outputMap); providers.onOutput(() => providers && render(providers.outputMap)); });
  }
  function hide() {
    open = false; hiddenAt = Date.now();
    $(PANEL[mode]).classList.add('closing'); arm(null);
    providers?.stopAll(); providers = null;
    setTimeout(() => { if (!open) win.hide(); }, D.closeMs ?? 160);
  }
  // clicking the bar already blurred (and hid) us, so the same button within 400 ms means "close"
  addEventListener('storage', e => {
    const m = { 'rs.dashCmd': 'dash', 'rs.powerCmd': 'power', 'rs.keysCmd': 'keys' }[e.key];
    if (!m) return;
    if (open) mode === m ? hide() : show(m);
    else if (mode !== m || Date.now() - hiddenAt > 400) show(m);
  });
  win.onFocusChanged(({ payload }) => { if (!payload && open && Date.now() - shownAt > 300) hide(); });
  addEventListener('keydown', e => { if (e.key === 'Escape' && open) hide(); });
  win.hide();

  function fill() {
    const i = +(store('rs.ws') ?? 0) || 0;
    $('artImg').src = `./art/card-${i % 5}.jpg`;
    set($('artK'), 'textContent', wsName(i));
    set($('artEn'), 'textContent', wsLabel(V, i));
    set($('precept'), 'textContent', D.mottos[Math.floor(Math.random() * D.mottos.length)]);
    clock(); calendar();
    for (const r of D.rings) fx.dashRing?.(r[0], 0, true);   // rings sweep in again
    fx.dashOpen?.(i);
  }
  function clock() {
    if (!open) return;
    const d = new Date(), [a, b, c] = D.date(d);
    set($('hm'), 'textContent', a); set($('jdate'), 'textContent', b); set($('edate'), 'textContent', c);
  }
  setInterval(clock, 1000);
  function calendar() {
    const now = new Date(), y = now.getFullYear(), m = now.getMonth();
    const first = new Date(y, m, 1).getDay(), days = new Date(y, m + 1, 0).getDate(), prev = new Date(y, m, 0).getDate();
    let h = `<div class="mh">${esc(D.month(y, m))}</div>` + D.wd.map((k, i) => `<div class="wd${i ? '' : ' sun'}">${esc(k)}</div>`).join('');
    const cells = Math.ceil((first + days) / 7) * 7;
    for (let c = 0; c < cells; c++) {
      const n = c - first + 1, out = n < 1 || n > days, num = out ? (n < 1 ? prev + n : n - days) : n;
      h += `<i class="${out ? 'out' : ''}${n === now.getDate() ? ' today' : ''}${c % 7 ? '' : ' sun'}">${D.num ? D.num(num) : num}</i>`;
    }
    $('cal').innerHTML = h;
  }
  // rings: the theme draws them (fx.dashRing(id, value, reset)); the engine fills the labels
  $('rings').innerHTML = D.rings.map(([id, k, en]) => fx.dashRingHTML ? fx.dashRingHTML(id, k, en) :
    `<div class="ring2" id="g-${id}"><div class="rw"><svg viewBox="0 0 60 60" id="r-${id}"></svg><em id="v-${id}">–</em></div><b>${esc(k)}</b><span id="s-${id}">${esc(en)}</span></div>`).join('');
  function ring(id, v, sub) {
    if (v == null || isNaN(v)) return;
    fx.dashRing?.(id, Math.min(v, 100), false);
    set($(`v-${id}`), 'textContent', String(Math.round(v)));
    if (sub) set($(`s-${id}`), 'textContent', sub);
  }
  let track = null;
  function render(o) {
    if (o.cpu) ring('cpu', o.cpu.usage);
    if (o.memory) ring('ram', o.memory.usage, `${(o.memory.usedMemory / 2 ** 30).toFixed(1)} G`);
    const disks = o.disk?.disks || [], d = disks.find(x => /^C:/i.test(x.mountPoint || '')) || disks[0];
    if (d) ring('disk', 100 * (1 - d.availableSpace.bytes / d.totalSpace.bytes), `${Math.round(d.availableSpace.bytes / 2 ** 30)} G free`);
    if (o.battery) {
      const b = o.battery, t = b.timeTillEmpty;
      ring('bat', b.chargePercent, b.isCharging ? 'charging' : t ? `${Math.floor(t / 3.6e6)}h ${Math.round(t / 6e4) % 60}m left` : null);
    } else $('g-bat')?.classList.add('none');
    const s = o.media?.currentSession;
    $('media').classList.toggle('idle', !s);
    $('media').classList.toggle('playing', !!s?.isPlaying);
    if (!s) { set($('mt'), 'textContent', D.idleMedia[0]); set($('ma'), 'textContent', D.idleMedia[1]); track = null; progress(); return; }
    set($('mt'), 'textContent', s.title || 'Untitled');
    set($('ma'), 'textContent', [s.artist, s.albumTitle].filter(Boolean).join(' · '));
    $('ppi')?.setAttribute('d', s.isPlaying ? 'M7 5h3.5v14H7zM13.5 5H17v14h-3.5z' : 'M7 5l12 7-12 7z');
    const len = (s.endTime || 0) - (s.startTime || 0);
    track = len > 0 ? { pos: (s.position || 0) - (s.startTime || 0), at: Date.now(), len, playing: s.isPlaying } : null;
    progress();
  }
  function progress() {
    const p = track ? Math.min(1, (track.pos + (track.playing ? (Date.now() - track.at) / 1000 : 0)) / track.len) : 0;
    $('mprog')?.style.setProperty('--p', p);
  }
  setInterval(() => open && progress(), 1000);
  const media = () => providers?.outputMap.media;
  click('pp', () => media()?.togglePlayPause());
  click('prev', () => media()?.previous());
  click('next', () => media()?.next());

  // power: full paths and exact arguments; the pack's privileges allow only these
  const POWER = {
    lock: [P.rundll32, 'user32.dll,LockWorkStation'],
    sleep: [P.powershell, "-NoProfile -NonInteractive -Command Add-Type -AssemblyName System.Windows.Forms;[System.Windows.Forms.Application]::SetSuspendState('Suspend',$false,$false)"],
    restart: [P.shutdown, '/r /t 0', true],
    off: [P.shutdown, '/s /t 0', true],
  };
  let armed = null, armTimer = 0;
  const label = ([k, l, h]) => { set($('pk'), 'textContent', k); set($('pl'), 'textContent', l); set($('ph'), 'textContent', h); };
  function arm(a) {
    clearTimeout(armTimer); armed = a;
    const p = $('power'); p.classList.remove('armed');
    for (const b of p.querySelectorAll('.pb')) b.classList.toggle('armed', b.dataset.a === a);
    if (!a) return label(D.power.idle);
    void p.offsetWidth; p.classList.add('armed');   // restarts the 3 s unwind
    label([D.power[a][0], D.power[a][1], D.power.confirm]);
    armTimer = setTimeout(() => arm(null), 3000);
  }
  for (const b of document.querySelectorAll('.pb')) {
    const a = b.dataset.a, [prog, args, confirm] = POWER[a], [k, l] = D.power[a];
    b.setAttribute('aria-label', l);
    if (!b.textContent.trim()) b.innerHTML = `<span class="k">${esc(k)}</span>`;
    b.onpointerenter = b.onfocus = () => armed || label([k, l, confirm ? D.power.twice : '']);
    b.onpointerleave = b.onblur = () => armed || label(D.power.idle);
    b.onclick = () => {
      if (confirm && armed !== a) return arm(a);
      hide();
      setTimeout(() => zebar.shellExec(prog, args).catch(err => console.error('power', err)), 180);   // let the panel go first
    };
  }

  // wallpaper picker: the theme's wallpapers (its five pieces of art, or the pictures you gave it), then the one made
  // from your pictures
  const WALLS = ART.walls.length ? [...ART.walls.map((en, i) => [D.wallKey ? D.wallKey(i) : String(i + 1), en]), D.walls[D.walls.length - 1]] : D.walls;
  const YOURS = String(WALLS.length - 1);
  $('wall').querySelector('.cards').innerHTML = WALLS.map(([k, en], i) =>
    `<button class="wc" data-w="${i}"><img src="./art/card-${i}.jpg" alt=""><span class="cap"><b class="k">${esc(k)}</b><i>${esc(en)}</i></span><em class="k">${esc(D.wallOn)}</em></button>`).join('');
  function markWall() {
    const cur = store('rs.wall') ?? YOURS;
    for (const c of document.querySelectorAll('.wc')) c.classList.toggle('on', c.dataset.w === cur);
    set($('wmsg'), 'textContent', '');
  }
  click('wallBtn', () => show('wall'));
  for (const c of document.querySelectorAll('.wc')) c.onclick = async () => {
    if (document.querySelector('.wc.busy')) return;
    c.classList.add('busy'); set($('wmsg'), 'textContent', D.wallBusy);
    try {
      const r = await zebar.shellExec(P.studio, `--wallpaper ${c.dataset.w}`);
      if (r.code) throw new Error(r.stderr || `exit ${r.code}`);
      store('rs.wall', c.dataset.w); markWall();
      setTimeout(() => open && mode === 'wall' && hide(), 700);
    } catch (err) { console.error('wall', err); set($('wmsg'), 'textContent', D.wallFail); }
    c.classList.remove('busy');
  };

  // key sheet: the bindings are the same in every theme; the theme names the sections and the bar parts
  const K = D.keys, row = ([a, b]) => `<dt>${esc(a)}</dt><dd>${esc(b)}</dd>`;
  const sec = ([k, en, hint], rows) => `<h3><b class="k">${esc(k)}</b> ${esc(en)}${hint ? ` <i>${esc(hint)}</i>` : ''}</h3><dl>${rows.map(row).join('')}</dl>`;
  const n = WS.length;
  $('keysheet').querySelector('.cols').innerHTML = `<div>${sec(K.windows, [
    ['H J K L', 'focus ← ↓ ↑ →'], ['Home / End', 'first / last column'], ['⇧ H / ⇧ L', 'move column ← →'],
    ['⇧ K / ⇧ J', 'move window ↑ ↓ in column'], ['[ / ]', 'window into column ← →'], [', / .', 'pull window in from ← →'],
    ['⇧ [ / ⇧ ]', 'push window out ← →'], ['- / =', 'narrower / wider'], ['0', 'equal widths (⇧ 0 heights)'],
    ['C / M', 'centre / maximize column'], ['F / ⇧ F', 'float / fullscreen'], ['T', 'tabbed column'], ['W', 'close window']])}</div>` +
    `<div>${sec(K.workspaces, [[`1 … ${n}`, WS.join(' · ')], [`⇧ 1 … ${n}`, 'send window there'], ['⇧ ← / ⇧ →', 'previous / next'],
      ['Space', 'overview'], ['S / ⇧ S', 'scratchpad show / stash'], ['Y', 'sticky (all workspaces)'], ['P', 'pause tiling'],
      ['R / ⇧ R', 're-tile / reload config']])}${sec(K.escape, [['Alt Space', 'Flow: apps, files, web'], ['Ctrl Alt ⇧ Q', 'rice off, taskbar back'],
      ['Win Ctrl Esc', 'untile every window']])}</div>` +
    `<div>${sec(K.bar, K.barRows)}</div>`;
  fx.dashReady?.();
}

/* ═══ desk widget: sits on the wallpaper under every window, ignores the mouse, wakes once a minute ═══════ */
export function desk(V) {
  zebar.currentWidget().tauriWindow.setIgnoreCursorEvents(true);
  const wall = () => { document.body.dataset.wall = store('rs.wall') ?? '5'; };
  const still = () => document.body.classList.toggle('nowm', store('rs.nowm') === '1');   // game mode: no motion on the desk
  const wx = () => { const [k, c] = (store('rs.wx') || '').split('|'); if (k) V.fx?.deskWx?.(k, +c); };
  wall(); wx(); still();
  addEventListener('storage', e => { if (e.key === 'rs.wall') wall(); if (e.key === 'rs.nowm') still(); if (e.key === 'rs.ws') V.fx?.deskWs?.(+e.newValue || 0); if (e.key === 'rs.wx') wx(); });
  let last = '';
  function tick() {
    const d = new Date(), hm = d.toTimeString().slice(0, 5);
    if (hm !== last) { last = hm; V.desk(d); }
    setTimeout(tick, 60000 - (Date.now() % 60000) + 20);
  }
  tick();
}
