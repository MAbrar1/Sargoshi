#!/usr/bin/env node
// Supply-chain check (Prompt B hard rule 7): every crates.io package in Cargo.lock must have
// been published at least N days ago (default 14). Needs network access to crates.io.
//   node tools/check-dep-age.mjs [--days 14]
import fs from 'node:fs';
import path from 'node:path';
import { ROOT, arg } from './lib.mjs';

const days = Number(arg('days') ?? 14);
// The root workspace lockfile and the separate lockfile of the cargo-fuzz crate.
const lockPaths = ['Cargo.lock', 'core/fuzz/Cargo.lock'].map((p) => path.join(ROOT, p)).filter((p) => fs.existsSync(p));
if (lockPaths.length === 0) { console.log('check-dep-age: no Cargo.lock yet, nothing to check'); process.exit(0); }
const seen = new Set();
const pkgs = [];
for (const lockPath of lockPaths) {
  const lock = fs.readFileSync(lockPath, 'utf8');
  for (const m of lock.matchAll(/\[\[package\]\]\nname = "([^"]+)"\nversion = "([^"]+)"\n(?:source = "([^"]+)")?/g)) {
    if (m[3] && m[3].includes('crates.io') && !seen.has(`${m[1]}@${m[2]}`)) { seen.add(`${m[1]}@${m[2]}`); pkgs.push([m[1], m[2]]); }
  }
}
const now = Date.now();
const bad = [];
for (const [name, version] of pkgs) {
  let published = null;
  try {
    const res = await fetch(`https://crates.io/api/v1/crates/${name}/${version}`, { headers: { 'User-Agent': 'sargoshi-build (supply-chain verification)' } });
    if (res.ok) published = (await res.json()).version?.created_at ?? null;
  } catch { /* reported below */ }
  if (!published) { bad.push(`${name} ${version}: release date could not be verified`); }
  else {
    const age = (now - Date.parse(published)) / 86_400_000;
    if (age < days) bad.push(`${name} ${version}: published ${published.slice(0, 10)} (${age.toFixed(1)} days old, minimum ${days})`);
  }
  await new Promise((r) => setTimeout(r, 200));
}
if (bad.length) {
  console.error(`check-dep-age: ${bad.length} problem(s)\n` + bad.join('\n') + '\nFix: cargo update -p <name>@<version> --precise <older version>');
  process.exit(1);
}
console.log(`check-dep-age: OK (${pkgs.length} crates, all at least ${days} days old)`);
