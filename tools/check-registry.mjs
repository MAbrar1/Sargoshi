#!/usr/bin/env node
// Verifies the ledger against spec-data and the code. Exit code 1 on any problem.
//   node tools/check-registry.mjs                 consistency of everything
//   node tools/check-registry.mjs --phase P07B    additionally: every row of this phase and all earlier phases must be terminal
//   node tools/check-registry.mjs --final         every row of every phase must be terminal
import fs from 'node:fs';
import path from 'node:path';
import { ROOT, specItems, readStatus, collectTags, phaseIndex, PHASES, STATUSES, TERMINAL, DONE_LIKE, arg, flag } from './lib.mjs';

const problems = [];
const spec = specItems(); const specIds = new Set(spec.map((s) => s.id));
const status = readStatus();
if (!status.length) { console.error('status.csv missing or empty. Run: node tools/ledger.mjs init'); process.exit(1); }
const byId = new Map(status.map((r) => [r.id, r]));
for (const s of spec) if (!byId.has(s.id)) problems.push(`missing ledger row: ${s.id}`);
for (const r of status) if (!specIds.has(r.id)) problems.push(`ledger row not in spec-data: ${r.id}`);
const seen = new Set(); for (const r of status) { if (seen.has(r.id)) problems.push(`duplicate ledger row: ${r.id}`); seen.add(r.id); }
const tags = collectTags();
const specById = new Map(spec.map((s) => [s.id, s]));
const runs = fs.existsSync(path.join(ROOT, 'docs/quality/DEVICE-RUNS.md')) ? fs.readFileSync(path.join(ROOT, 'docs/quality/DEVICE-RUNS.md'), 'utf8') : '';
const DEVICE_MODULES = new Set(['VLT', 'CAL', 'SHR']);
const ext = fs.existsSync(path.join(ROOT, 'docs/ledger/EXTERNAL.md')) ? fs.readFileSync(path.join(ROOT, 'docs/ledger/EXTERNAL.md'), 'utf8') : '';

for (const r of status) {
  const s = specById.get(r.id); if (!s) continue;
  if (!STATUSES.includes(r.status)) { problems.push(`${r.id}: invalid status "${r.status}"`); continue; }
  if (DONE_LIKE.has(r.status)) {
    const t = tags.get(r.id) || { impl: new Set(), test: new Set() };
    if (['REG', 'AI', 'ALR'].includes(r.kind) && !t.impl.size) problems.push(`${r.id}: marked ${r.status} but no implementation tag SG:${r.id} found in code`);
    if (!t.test.size) problems.push(`${r.id}: marked ${r.status} but no test tag SG:${r.id} found in tests`);
    if (r.status === 'DONE' && (r.kind === 'LKV' || (r.kind === 'REG' && DEVICE_MODULES.has(s.module))) && !new RegExp(r.id + '[^\\n]*PASS').test(runs)) problems.push(`${r.id}: needs a real device or emulator run. Add a line "${r.id} | device | android | command | PASS" to docs/quality/DEVICE-RUNS.md after actually running it`);
    if (r.status === 'DONE_NEEDS_CREDENTIALS' && !ext.includes(r.id)) problems.push(`${r.id}: DONE_NEEDS_CREDENTIALS but docs/ledger/EXTERNAL.md does not list it`);
  }
  if (['DESIGN_ONLY', 'DEFERRED'].includes(r.status) && s.bp !== 'v2') problems.push(`${r.id}: ${s.bp} row cannot be ${r.status}`);
  if (['DESIGN_ONLY', 'DEFERRED', 'EXTERNAL_BLOCKED'].includes(r.status) && (r.note || '').length < 30) problems.push(`${r.id}: ${r.status} needs a reason of at least 30 characters`);
  if (r.status === 'EXTERNAL_BLOCKED' && !ext.includes(r.id)) problems.push(`${r.id}: EXTERNAL_BLOCKED but docs/ledger/EXTERNAL.md does not list it`);
}
// tags that reference ids that do not exist (typos)
for (const id of tags.keys()) if (!specIds.has(id)) problems.push(`code references unknown id tag SG:${id}`);

const gatePhase = flag('final') ? PHASES[PHASES.length - 1] : arg('phase');
if (gatePhase) {
  const limit = phaseIndex(gatePhase);
  if (limit < 0) problems.push(`unknown phase ${gatePhase}`);
  for (const r of status) if (phaseIndex(r.phase) <= limit && !TERMINAL.has(r.status)) problems.push(`phase gate ${gatePhase}: ${r.id} (${r.kind}, ${r.phase}) is ${r.status}`);
}
if (problems.length) {
  console.error(`check-registry: ${problems.length} problem(s)`);
  console.error(problems.slice(0, 60).join('\n') + (problems.length > 60 ? `\n... and ${problems.length - 60} more` : ''));
  process.exit(1);
}
console.log(`check-registry: OK (${status.length} ledger rows${gatePhase ? `, phase gate ${gatePhase} passed` : ''})`);
