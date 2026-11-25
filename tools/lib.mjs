// Shared helpers for Sargoshi tooling. Node >= 18, no dependencies.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
export const SPEC = path.join(ROOT, 'docs', 'spec-data');
export const LEDGER_DIR = path.join(ROOT, 'docs', 'ledger');
export const STATUS_CSV = path.join(LEDGER_DIR, 'status.csv');

export const PHASES = ['P00','P01','P02','P03','P04','P05','P06','P07A','P07B','P07C','P08','P09','P10','P11','P12','P13','P14A','P14B','P15','P16','P17','P18','P19','P20','P21','P22','P23'];
export const phaseIndex = (p) => PHASES.indexOf(p);
export const STATUSES = ['NOT_STARTED','IN_PROGRESS','DONE','DONE_NEEDS_CREDENTIALS','DESIGN_ONLY','DEFERRED','EXTERNAL_BLOCKED'];
export const TERMINAL = new Set(['DONE','DONE_NEEDS_CREDENTIALS','DESIGN_ONLY','DEFERRED','EXTERNAL_BLOCKED']);
export const DONE_LIKE = new Set(['DONE','DONE_NEEDS_CREDENTIALS']);

export function parseCSV(text) {
  const rows = []; let row = []; let cell = ''; let q = false;
  for (let i = 0; i < text.length; i++) {
    const c = text[i];
    if (q) {
      if (c === '"') { if (text[i + 1] === '"') { cell += '"'; i++; } else q = false; }
      else cell += c;
    } else if (c === '"') q = true;
    else if (c === ',') { row.push(cell); cell = ''; }
    else if (c === '\n') { row.push(cell); rows.push(row); row = []; cell = ''; }
    else if (c === '\r') { /* skip */ }
    else cell += c;
  }
  if (cell.length || row.length) { row.push(cell); rows.push(row); }
  return rows;
}
export function toCSV(rows) {
  const esc = (v) => { v = String(v ?? ''); return /[",\n\r]/.test(v) ? '"' + v.replace(/"/g, '""') + '"' : v; };
  return rows.map((r) => r.map(esc).join(',')).join('\n') + '\n';
}
export function readTable(file) {
  const rows = parseCSV(fs.readFileSync(file, 'utf8')).filter((r) => r.length > 1 || (r[0] ?? '') !== '');
  const head = rows[0];
  return rows.slice(1).map((r) => Object.fromEntries(head.map((h, i) => [h, r[i] ?? ''])));
}
export const readSpec = (name) => readTable(path.join(SPEC, name));

// Everything the ledger tracks, derived from spec-data. kind drives the evidence rule.
export function specItems() {
  const items = [];
  for (const r of readSpec('registry.csv')) items.push({ id: r.id, kind: 'REG', module: r.module, phase: r.build_phase, text: r.feature, line: r.blueprint_line, bp: r.blueprint_phase, v2: r.v2_mode });
  for (const r of readSpec('ai-features.csv')) items.push({ id: r.id, kind: 'AI', phase: r.build_phase, text: r.feature, line: r.blueprint_line, bp: r.v2_mode ? 'v2' : 'v1', v2: r.v2_mode ? 'TRY_BUILD' : '' });
  for (const r of readSpec('alerts.csv')) items.push({ id: r.id, kind: 'ALR', phase: r.build_phase, text: r.trigger, line: r.blueprint_line, bp: 'v1', v2: '' });
  for (const r of readSpec('settings.csv')) items.push({ id: r.id, kind: 'ST', phase: settingPhase(r), text: `${r.group}: ${r.setting}`, line: r.blueprint_line, bp: 'v1', v2: '' });
  const jseen = new Set();
  for (const r of readSpec('journeys.csv')) if (!jseen.has(r.id)) { jseen.add(r.id); items.push({ id: r.id, kind: 'JNY', phase: r.build_phase, text: r.journey, line: '', bp: 'v1', v2: '' }); }
  for (const r of readSpec('leak-matrix.csv')) items.push({ id: r.id, kind: 'LKV', phase: r.build_phase, text: r.leak_vector, line: '', bp: 'v1', v2: '' });
  for (const r of readSpec('components.csv')) items.push({ id: r.id, kind: 'CMP', phase: r.build_phase, text: r.component, line: '', bp: 'v1', v2: '' });
  for (const r of readSpec('screens.csv')) items.push({ id: r.id, kind: 'SCR', phase: r.build_phase, text: r.screen, line: '', bp: 'v1', v2: '' });
  return items;
}
// A setting becomes real in the phase that builds the feature it controls; fall back to P18.
let _regPhase = null;
function settingPhase(r) {
  if (r.group_letter === 'I') return 'P17';
  if (!_regPhase) _regPhase = Object.fromEntries(readSpec('registry.csv').map((x) => [x.id, x.build_phase]));
  const ids = (r.feature.match(/[A-Z]{3}-\d{2}/g) || []).filter((i) => _regPhase[i]);
  const known = ids.filter((i) => _regPhase[i] !== 'P21').map((i) => _regPhase[i]);
  if (!known.length) return /AI-\d/.test(r.feature) ? 'P17' : 'P18';
  return known.sort((a, b) => phaseIndex(b) - phaseIndex(a))[0]; // latest feature phase
}

export function readStatus() {
  if (!fs.existsSync(STATUS_CSV)) return [];
  return readTable(STATUS_CSV);
}
export function writeStatus(rows) {
  const head = ['id', 'kind', 'phase', 'status', 'note', 'updated'];
  fs.writeFileSync(STATUS_CSV, toCSV([head, ...rows.map((r) => head.map((h) => r[h] ?? ''))]));
}

export function walk(dir, { skip = [], exts = null } = {}) {
  const out = [];
  if (!fs.existsSync(dir)) return out;
  const SKIP = new Set(['node_modules', '.git', 'build', '.dart_tool', 'target', '.gradle', '.idea', 'Pods', ...skip]);
  const rec = (d) => {
    for (const e of fs.readdirSync(d, { withFileTypes: true })) {
      if (SKIP.has(e.name)) continue;
      const p = path.join(d, e.name);
      if (e.isDirectory()) rec(p);
      else if (!exts || exts.includes(path.extname(e.name))) out.push(p);
    }
  };
  rec(dir);
  return out;
}
export const rel = (p) => path.relative(ROOT, p).split(path.sep).join('/');

export const CODE_EXTS = ['.dart', '.kt', '.kts', '.rs', '.java', '.ts', '.js', '.mjs', '.html', '.toml', '.sql', '.xml', '.yaml', '.yml', '.gradle'];
export const isTestPath = (p) => /(^|\/)(test|tests|integration_test|androidTest|testing|golden)(\/|$)|_test\.(dart|rs|kt)$|Test\.kt$/.test(p);
export const SOURCE_ROOTS = ['apps', 'core', 'server', 'infra', 'tools'];

// Collect every SG:<ID> tag in source files. Returns Map(id -> {impl:Set(files), test:Set(files)})
export function collectTags() {
  const map = new Map();
  const re = /SG:([A-Z]{2,4}-\d{2,3}(?:-\d{2})?)/g;
  for (const root of SOURCE_ROOTS) {
    for (const f of walk(path.join(ROOT, root), { exts: CODE_EXTS })) {
      const r = rel(f);
      if (r.startsWith('tools/')) continue;
      let txt; try { txt = fs.readFileSync(f, 'utf8'); } catch { continue; }
      let m; re.lastIndex = 0;
      while ((m = re.exec(txt))) {
        const e = map.get(m[1]) || { impl: new Set(), test: new Set() };
        (isTestPath(r) ? e.test : e.impl).add(r);
        map.set(m[1], e);
      }
    }
  }
  return map;
}
export function currentPhase() {
  try {
    const t = fs.readFileSync(path.join(LEDGER_DIR, 'PROGRESS.md'), 'utf8');
    const m = t.match(/^CURRENT_PHASE:\s*(P\d{2}[A-C]?)/m);
    if (m && PHASES.includes(m[1])) return m[1];
  } catch {}
  return 'P00';
}
export const today = () => new Date().toISOString().slice(0, 10);
export function arg(name, def = null) {
  const i = process.argv.indexOf('--' + name);
  return i >= 0 ? process.argv[i + 1] : def;
}
export const flag = (name) => process.argv.includes('--' + name);
