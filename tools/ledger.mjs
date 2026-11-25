#!/usr/bin/env node
// Ledger CLI. The ONLY supported way to change docs/ledger/status.csv.
//   node tools/ledger.mjs init                       (re)build status.csv from spec-data, keeping existing statuses
//   node tools/ledger.mjs status [--phase P07A]      counts per kind and status
//   node tools/ledger.mjs next [--phase P07A] [--kind REG|AI|ALR|ST|JNY|LKV|CMP|SCR] [--n 8] next rows to work on, with spec text
//   node tools/ledger.mjs start ID...                mark IN_PROGRESS
//   node tools/ledger.mjs claim ID... [--credentials] [--note "..."]   verify evidence tags then mark DONE
//   node tools/ledger.mjs waive ID... --status DESIGN_ONLY|DEFERRED|EXTERNAL_BLOCKED --reason "..." --doc docs/...
import fs from 'node:fs';
import path from 'node:path';
import { ROOT, specItems, readStatus, writeStatus, collectTags, currentPhase, phaseIndex, PHASES, TERMINAL, DONE_LIKE, arg, flag, today } from './lib.mjs';

const [cmd, ...rest] = process.argv.slice(2);
const ids = rest.filter((a, i) => !a.startsWith('--') && !(rest[i - 1] || '').startsWith('--'));
const fail = (m) => { console.error('LEDGER ERROR: ' + m); process.exit(1); };

function load() {
  const spec = specItems();
  const cur = new Map(readStatus().map((r) => [r.id, r]));
  const rows = spec.map((s) => cur.get(s.id) ? { ...cur.get(s.id), kind: s.kind, phase: s.phase } : { id: s.id, kind: s.kind, phase: s.phase, status: 'NOT_STARTED', note: '', updated: '' });
  return { spec, rows, byId: new Map(rows.map((r) => [r.id, r])), specById: new Map(spec.map((s) => [s.id, s])) };
}

if (cmd === 'init') {
  const { rows } = load(); writeStatus(rows);
  console.log(`status.csv written: ${rows.length} rows`);
} else if (cmd === 'status') {
  const { rows } = load(); const ph = arg('phase');
  const sel = ph ? rows.filter((r) => r.phase === ph) : rows;
  const kinds = [...new Set(sel.map((r) => r.kind))];
  console.log(['kind', ...['NOT_STARTED', 'IN_PROGRESS', 'DONE', 'DONE_NEEDS_CREDENTIALS', 'DESIGN_ONLY', 'DEFERRED', 'EXTERNAL_BLOCKED'].map((s) => s.slice(0, 8)), 'total'].join('\t'));
  for (const k of kinds) {
    const g = sel.filter((r) => r.kind === k);
    console.log([k, ...['NOT_STARTED', 'IN_PROGRESS', 'DONE', 'DONE_NEEDS_CREDENTIALS', 'DESIGN_ONLY', 'DEFERRED', 'EXTERNAL_BLOCKED'].map((s) => g.filter((r) => r.status === s).length), g.length].join('\t'));
  }
  const open = sel.filter((r) => !TERMINAL.has(r.status)).length;
  console.log(`\nopen rows${ph ? ' in ' + ph : ''}: ${open} of ${sel.length}   current phase: ${currentPhase()}`);
} else if (cmd === 'next') {
  const { rows, specById } = load(); const ph = arg('phase', currentPhase()); const n = Number(arg('n', '8'));
  const kd = arg('kind'); const list = rows.filter((r) => r.phase === ph && !TERMINAL.has(r.status) && (!kd || r.kind === kd)).slice(0, n);
  if (!list.length) console.log(`No open rows in ${ph}. Run: node tools/check-registry.mjs --phase ${ph}`);
  for (const r of list) { const s = specById.get(r.id); console.log(`${r.id} [${r.kind}] ${r.status}  ${s.text}${s.line ? `   (blueprint line ${s.line})` : ''}${s.v2 ? `   v2_mode=${s.v2}` : ''}`); }
} else if (cmd === 'start') {
  const { rows, byId } = load();
  for (const id of ids) { const r = byId.get(id); if (!r) fail('unknown id ' + id); if (r.status === 'NOT_STARTED') { r.status = 'IN_PROGRESS'; r.updated = today(); } }
  writeStatus(rows); console.log('started', ids.join(' '));
} else if (cmd === 'claim') {
  const { rows, byId } = load(); const tags = collectTags(); const problems = [];
  for (const id of ids) {
    const r = byId.get(id); if (!r) { problems.push(`${id}: unknown id`); continue; }
    const t = tags.get(id) || { impl: new Set(), test: new Set() };
    const needImpl = ['REG', 'AI', 'ALR'].includes(r.kind);
    if (needImpl && t.impl.size === 0) problems.push(`${id}: no implementation file contains the tag "SG:${id}"`);
    if (t.test.size === 0) problems.push(`${id}: no test file contains the tag "SG:${id}"`);
  }
  if (problems.length) { console.error(problems.join('\n')); fail('claim refused. Add the tags where the real code and real tests are, then retry.'); }
  for (const id of ids) { const r = byId.get(id); r.status = flag('credentials') ? 'DONE_NEEDS_CREDENTIALS' : 'DONE'; r.note = arg('note', r.note || ''); r.updated = today(); }
  writeStatus(rows); console.log(`claimed ${ids.length}: ${ids.join(' ')}`);
} else if (cmd === 'waive') {
  const status = arg('status'); const reason = arg('reason', ''); const doc = arg('doc', '');
  if (!['DESIGN_ONLY', 'DEFERRED', 'EXTERNAL_BLOCKED'].includes(status)) fail('--status must be DESIGN_ONLY, DEFERRED or EXTERNAL_BLOCKED');
  if (reason.length < 30) fail('--reason must be at least 30 characters and explain exactly why');
  if (!doc || !fs.existsSync(path.join(ROOT, doc))) fail('--doc must be an existing file under the repo that documents this decision');
  const { rows, byId, specById } = load(); const docText = fs.readFileSync(path.join(ROOT, doc), 'utf8');
  for (const id of ids) {
    const r = byId.get(id); if (!r) fail('unknown id ' + id);
    const s = specById.get(id);
    if (status !== 'EXTERNAL_BLOCKED' && !(s.kind === 'REG' || s.kind === 'AI') ) fail(`${id}: only v2 registry rows may be ${status}`);
    if (status !== 'EXTERNAL_BLOCKED' && s.bp !== 'v2') fail(`${id}: is ${s.bp}, not v2. MVP and v1 rows cannot be ${status}; build it or mark EXTERNAL_BLOCKED with a real external reason`);
    if (!docText.includes(id)) fail(`${id}: ${doc} must mention the id`);
    r.status = status; r.note = reason; r.updated = today();
  }
  writeStatus(rows); console.log(`waived ${ids.length} as ${status}`);
} else {
  console.log(fs.readFileSync(new URL(import.meta.url), 'utf8').split('\n').slice(1, 9).join('\n'));
}
