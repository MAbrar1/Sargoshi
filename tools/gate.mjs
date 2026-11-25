#!/usr/bin/env node
// The single command that decides whether work is acceptable.
//   node tools/gate.mjs              project checks (tree, reality, registry consistency)
//   node tools/gate.mjs --full       plus toolchain checks for whatever exists (flutter, cargo, gradle)
//   node tools/gate.mjs --phase P07B plus the phase gate: every row of that phase and earlier must be terminal
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { ROOT, arg, flag, currentPhase } from './lib.mjs';

const steps = [
  ['tree', 'node', ['tools/check-tree.mjs'], '.'],
  ['reality', 'node', ['tools/check-reality.mjs'], '.'],
  ['registry', 'node', ['tools/check-registry.mjs', ...(arg('phase') ? ['--phase', arg('phase')] : [])], '.'],
];
for (const g of fs.existsSync(path.join(ROOT, 'tools')) ? fs.readdirSync(path.join(ROOT, 'tools')).filter((n) => /^gen-.*\.mjs$/.test(n)) : []) steps.push([`${g} --check (generated files up to date)`, 'node', [`tools/${g}`, '--check'], '.']);
if (flag('full') || arg('phase')) {
  const has = (p) => fs.existsSync(path.join(ROOT, p));
  if (has('apps/mobile/pubspec.yaml')) {
    steps.push(['flutter analyze', 'flutter', ['analyze', '--fatal-infos'], 'apps/mobile']);
    steps.push(['flutter test', 'flutter', ['test'], 'apps/mobile']);
  }
  if (has('Cargo.toml')) {
    steps.push(['cargo fmt', 'cargo', ['fmt', '--all', '--', '--check'], '.']);
    steps.push(['cargo clippy', 'cargo', ['clippy', '--workspace', '--all-targets', '--', '-D', 'warnings'], '.']);
    steps.push(['cargo test', 'cargo', ['test', '--workspace'], '.']);
    if (has('deny.toml')) steps.push(['cargo deny', 'cargo', ['deny', 'check'], '.']);
  }
}
let failed = 0; const results = [];
for (const [name, cmd, args, cwd] of steps) {
  process.stdout.write(`\n=== ${name} ===\n`);
  const r = spawnSync(cmd, args, { cwd: path.join(ROOT, cwd), stdio: 'inherit', shell: process.platform === 'win32' });
  const ok = r.status === 0; if (!ok) failed++;
  results.push(`${ok ? 'PASS' : 'FAIL'}  ${name}${r.error ? '  (' + r.error.message + ')' : ''}`);
}
console.log('\n--- GATE SUMMARY (phase ' + currentPhase() + ') ---\n' + results.join('\n'));
if (failed) { console.error(`\nGATE FAILED: ${failed} step(s). Fix them. Do not claim completion.`); process.exit(1); }
console.log('\nGATE PASSED');
