import * as zebar from './lib/zebar.js';
const LWM = {{lwm}};
const ROMAN = {{roman}};
const $ = id => document.getElementById(id);
const set = (id, v) => { const el = $(id); if (el && el.textContent !== v) el.textContent = v; };
const roman = n => { if (!n) return 'N'; let s = ''; for (const [v, r] of [[50,'L'],[40,'XL'],[10,'X'],[9,'IX'],[5,'V'],[4,'IV'],[1,'I']]) while (n >= v) { s += r; n -= v; } return s; };
const DAYS = ['SUN','MON','TUE','WED','THU','FRI','SAT'], MON = ['JAN','FEB','MAR','APR','MAY','JUN','JUL','AUG','SEP','OCT','NOV','DEC'];
const p = zebar.createProviderGroup({
  cpu: { type: 'cpu', refreshInterval: 5000 }, memory: { type: 'memory', refreshInterval: 10000 },
  battery: { type: 'battery', refreshInterval: 30000 }, date: { type: 'date', formatting: 'HH:mm', refreshInterval: 1000 },
});
p.onOutput(() => {
  const o = p.outputMap;
  if (o.cpu) set('cpu', Math.round(o.cpu.usage) + '%');
  if (o.memory) set('mem', Math.round(o.memory.usage) + '%');
  if (o.battery) set('bat', Math.round(o.battery.chargePercent) + '%'); else { const b = $('batw'); if (b) b.style.display = 'none'; }
  if (o.date) {
    const [h, m] = o.date.formatted.split(':').map(Number);
    set('clock', ROMAN ? roman(h) + ' \u00B7 ' + roman(m) : o.date.formatted);
    const d = new Date(); set('date', DAYS[d.getDay()] + ' ' + d.getDate() + ' ' + MON[d.getMonth()]);
  }
});
let active = 0, occupied = new Set([0]), building = null, up = false, retry = 5000;
const ws = [...document.querySelectorAll('.w')];
const stamp = () => ws.forEach((b, i) => { b.classList.toggle('on', i === active); b.classList.toggle('occ', occupied.has(i)); });
ws.forEach((b, i) => b.onclick = () => up && zebar.shellExec(LWM, 'workspace ' + (i + 1)).catch(() => {}));
function onEvent(e) {
  if (e.type === 'workspace_changed') { active = e.new_index; stamp(); }
  else if (e.type === 'workspace_snapshot_begin') building = new Set();
  else if (e.type === 'workspace_snapshot_chunk') for (const r of e.records || []) {
    if (r.kind === 'monitor') active = r.active_workspace_index;
    if (r.kind === 'window' && building) building.add(r.workspace_index);
  }
  else if (e.type === 'workspace_snapshot_end') { if (building) occupied = building; building = null; stamp(); }
}
const again = () => { up = false; setTimeout(subscribe, retry); retry = Math.min(retry * 2, 60000); };
async function subscribe() {
  let buf = '';
  const proc = await zebar.shellSpawn(LWM, 'subscribe --events workspace,workspace_state');
  up = true;
  proc.onStdout(chunk => {
    retry = 5000; buf += chunk; let i;
    while ((i = buf.indexOf('\n')) >= 0) { const line = buf.slice(0, i).trim(); buf = buf.slice(i + 1); if (line.startsWith('{')) try { onEvent(JSON.parse(line)); } catch {} }
  });
  proc.onExit(again);
}
stamp(); subscribe().catch(again);
