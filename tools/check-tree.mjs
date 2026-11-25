#!/usr/bin/env node
// Enforces docs/ledger/PROJECT-TREE.md using tools/tree-spec.json.
import fs from 'node:fs';
import path from 'node:path';
import { ROOT, currentPhase, phaseIndex } from './lib.mjs';

const spec = JSON.parse(fs.readFileSync(path.join(ROOT, 'tools', 'tree-spec.json'), 'utf8'));
const cur = phaseIndex(currentPhase());
const problems = [];
for (const e of spec.required) {
  if (phaseIndex(e.from) > cur) continue;
  const p = path.join(ROOT, e.path);
  if (!fs.existsSync(p)) problems.push(`missing ${e.type}: ${e.path} (required from ${e.from})`);
  else if ((e.type === 'dir') !== fs.statSync(p).isDirectory()) problems.push(`wrong type for ${e.path}`);
}
const names = (d) => fs.existsSync(path.join(ROOT, d)) ? fs.readdirSync(path.join(ROOT, d), { withFileTypes: true }).filter((x) => x.isDirectory()).map((x) => x.name) : [];
for (const [dir, allowed] of Object.entries(spec.allowedChildren)) {
  for (const n of names(dir)) if (!allowed.includes(n)) problems.push(`unexpected directory ${dir}/${n} (allowed: ${allowed.join(', ')})`);
}
const rootEntries = fs.readdirSync(ROOT).filter((n) => n !== '.git' && n !== 'node_modules');
for (const n of rootEntries) if (!spec.allowedRoot.includes(n)) problems.push(`unexpected root entry: ${n}`);
// file naming
const walkAll = (d, acc = []) => { if (!fs.existsSync(d)) return acc; for (const e of fs.readdirSync(d, { withFileTypes: true })) { if (['.git', 'node_modules', 'build', '.dart_tool', 'target', '.gradle', '.idea', 'docs'].includes(e.name)) continue; const p = path.join(d, e.name); e.isDirectory() ? walkAll(p, acc) : acc.push(p); } return acc; };
for (const f of walkAll(path.join(ROOT, 'apps/mobile/lib')).concat(fs.existsSync(path.join(ROOT, 'apps/mobile/test')) ? walkAll(path.join(ROOT, 'apps/mobile/test')) : [])) {
  if (f.endsWith('.dart') && !/^[a-z0-9_]+(\.(g|freezed|mocks))?\.dart$/.test(path.basename(f))) problems.push(`dart file must be snake_case: ${path.relative(ROOT, f)}`);
}
// no stray junk anywhere
for (const f of walkAll(ROOT)) { const b = path.basename(f); if (/\.(orig|rej|bak|tmp|log)$|^~|\.DS_Store$|^Thumbs\.db$|copy\.\w+$| \(\d+\)\./i.test(b)) problems.push(`junk file: ${path.relative(ROOT, f)}`); }
if (problems.length) { console.error(`check-tree: ${problems.length} problem(s)\n` + problems.slice(0, 60).join('\n')); process.exit(1); }
console.log(`check-tree: OK (phase ${currentPhase()})`);
