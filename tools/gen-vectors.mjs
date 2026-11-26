#!/usr/bin/env node
// Regenerates core/crates/sg-testvectors/vectors/*.txt from upstream known-answer test data.
// Upstream commits are pinned below; SOURCES.md records the SHA-256 of every upstream file.
//   node tools/gen-vectors.mjs            (downloads from raw.githubusercontent.com)
//   node tools/gen-vectors.mjs --cache D  (reads <name>.json files from directory D instead)
//   node tools/gen-vectors.mjs --check    (offline: committed vector files match the hashes in SOURCES.md)
//   node tools/gen-vectors.mjs --check --online  (also re-downloads upstream and compares byte for byte)
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { ROOT, arg } from './lib.mjs';

const WYCHEPROOF = '65ca494727a52944c204441c860b039e6786b99b'; // C2SP/wycheproof, 2026-11-11
const ACVP = '975de31eb83d87039ec88934fdc47d8c312b892d'; // usnistgov/ACVP-Server master as of 25 Nov 2026
const OUT = path.join(ROOT, 'core/crates/sg-testvectors/vectors');
const cache = arg('cache');
const check = process.argv.includes('--check');
const online = process.argv.includes('--online');
const sources = [];
const outputs = new Map();
const sha = (b) => crypto.createHash('sha256').update(b).digest('hex');

if (check && !online) {
  const sourcesPath = path.join(OUT, 'SOURCES.md');
  if (!fs.existsSync(sourcesPath)) { console.error('gen-vectors --check: vectors/SOURCES.md is missing'); process.exit(1); }
  const recorded = new Map([...fs.readFileSync(sourcesPath, 'utf8').matchAll(/^\| (\S+\.txt) \| ([0-9a-f]{64}) \|/gm)].map((m) => [m[1], m[2]]));
  const onDisk = fs.readdirSync(OUT).filter((n) => n.endsWith('.txt'));
  const problems = [];
  for (const n of onDisk) {
    if (!recorded.has(n)) problems.push(`${n}: not listed in SOURCES.md`);
    else if (sha(fs.readFileSync(path.join(OUT, n))) !== recorded.get(n)) problems.push(`${n}: content differs from the hash in SOURCES.md`);
  }
  for (const n of recorded.keys()) if (!onDisk.includes(n)) problems.push(`${n}: listed in SOURCES.md but missing`);
  if (problems.length) { console.error('gen-vectors --check: ' + problems.length + ' problem(s)\n' + problems.join('\n') + '\nFix: node tools/gen-vectors.mjs'); process.exit(1); }
  console.log(`gen-vectors --check: OK (${onDisk.length} vector files match SOURCES.md)`);
  process.exit(0);
}

async function load(label, url, cacheName) {
  let buf;
  if (cache) buf = fs.readFileSync(path.join(cache, cacheName));
  else {
    const res = await fetch(url);
    if (!res.ok) throw new Error(`download failed ${res.status}: ${url}`);
    buf = Buffer.from(await res.arrayBuffer());
  }
  sources.push({ label, url, sha256: sha(buf), bytes: buf.length });
  return JSON.parse(buf.toString('utf8'));
}
const wy = (n) => load(`wycheproof/${n}`, `https://raw.githubusercontent.com/C2SP/wycheproof/${WYCHEPROOF}/testvectors_v1/${n}.json`, `${n}.json`);
const acvp = (dir, f) => load(`ACVP-Server/${dir}/${f}`, `https://raw.githubusercontent.com/usnistgov/ACVP-Server/${ACVP}/gen-val/json-files/${dir}/${f}.json`, `${dir}-${f}.json`);

const hex = (s) => (s && s.length ? s.toLowerCase() : '-');
const flags = (a) => (a && a.length ? a.join(',') : '-');
function write(name, header, rows) {
  outputs.set(name, `# ${header}\n` + rows.join('\n') + '\n');
  console.log(`${name}: ${rows.length} records`);
}

// --- Wycheproof
let d = await wy('hkdf_sha256_test');
write('hkdf_sha256.txt', 'tcId result ikm salt info size okm   (Wycheproof hkdf_sha256_test, size in bytes)',
  d.testGroups.flatMap((g) => g.tests.map((t) => [t.tcId, t.result, hex(t.ikm), hex(t.salt), hex(t.info), t.size, hex(t.okm)].join(' '))));

d = await wy('x25519_test');
write('x25519.txt', 'tcId result public private shared flags   (Wycheproof x25519_test)',
  d.testGroups.flatMap((g) => g.tests.map((t) => [t.tcId, t.result, hex(t.public), hex(t.private), hex(t.shared), flags(t.flags)].join(' '))));

d = await wy('chacha20_poly1305_test');
write('chacha20_poly1305.txt', 'tcId result key iv aad msg ct tag flags   (Wycheproof chacha20_poly1305_test, 256-bit key, 96-bit iv, 128-bit tag only)',
  d.testGroups.filter((g) => g.ivSize === 96 && g.keySize === 256 && g.tagSize === 128)
    .flatMap((g) => g.tests.map((t) => [t.tcId, t.result, hex(t.key), hex(t.iv), hex(t.aad), hex(t.msg), hex(t.ct), hex(t.tag), flags(t.flags)].join(' '))));

d = await wy('ed25519_test');
write('ed25519.txt', 'tcId result pk msg sig flags   (Wycheproof ed25519_test)',
  d.testGroups.flatMap((g) => g.tests.map((t) => [t.tcId, t.result, hex(g.publicKey.pk), hex(t.msg), hex(t.sig), flags(t.flags)].join(' '))));

// --- NIST ACVP, ML-KEM-768 only
const kg = await acvp('ML-KEM-keyGen-FIPS203', 'internalProjection');
write('mlkem768_keygen.txt', 'tcId d z ek dk   (NIST ACVP ML-KEM-keyGen-FIPS203, ML-KEM-768)',
  kg.testGroups.filter((g) => g.parameterSet === 'ML-KEM-768').flatMap((g) => g.tests.map((t) => [t.tcId, hex(t.d), hex(t.z), hex(t.ek), hex(t.dk)].join(' '))));

const ed = await acvp('ML-KEM-encapDecap-FIPS203', 'internalProjection');
const g768 = (fn) => ed.testGroups.filter((g) => g.parameterSet === 'ML-KEM-768' && g.function === fn).flatMap((g) => g.tests);
write('mlkem768_encaps.txt', 'tcId ek m c k   (NIST ACVP ML-KEM-encapDecap-FIPS203, encapsulation AFT)',
  g768('encapsulation').map((t) => [t.tcId, hex(t.ek), hex(t.m), hex(t.c), hex(t.k)].join(' ')));
write('mlkem768_decaps.txt', 'tcId dk c k   (NIST ACVP ML-KEM-encapDecap-FIPS203, decapsulation VAL)',
  g768('decapsulation').map((t) => [t.tcId, hex(t.dk), hex(t.c), hex(t.k)].join(' ')));
write('mlkem768_ek_check.txt', 'tcId ek passed   (NIST ACVP encapsulationKeyCheck)',
  g768('encapsulationKeyCheck').map((t) => [t.tcId, hex(t.ek), t.testPassed ? 1 : 0].join(' ')));
write('mlkem768_dk_check.txt', 'tcId dk passed   (NIST ACVP decapsulationKeyCheck)',
  g768('decapsulationKeyCheck').map((t) => [t.tcId, hex(t.dk), t.testPassed ? 1 : 0].join(' ')));

const generated = [...outputs.entries()].map(([name, text]) => `| ${name} | ${sha(text)} | ${Buffer.byteLength(text)} |`);
const sourcesText = `# Test vector sources

Generated by \`node tools/gen-vectors.mjs\`. Upstream commits are pinned; every upstream file's SHA-256 is recorded.
\`node tools/gen-vectors.mjs --check\` verifies the committed vector files against the second table below
(offline); add \`--online\` to re-download upstream and compare byte for byte.

- Project Wycheproof, https://github.com/C2SP/wycheproof, commit ${WYCHEPROOF} (Apache-2.0).
- NIST ACVP-Server, https://github.com/usnistgov/ACVP-Server, commit ${ACVP}. VERIFY: confirm the licence terms of the ACVP-Server repository before public release.

## Upstream files

| Upstream file | SHA-256 | Bytes |
|---|---|---|
${sources.map((s) => `| ${s.label} | ${s.sha256} | ${s.bytes} |`).join('\n')}

## Generated files

| Generated file | SHA-256 | Bytes |
|---|---|---|
${generated.join('\n')}
`;
if (check) {
  const problems = [];
  for (const [name, text] of outputs) {
    const file = path.join(OUT, name);
    if (!fs.existsSync(file) || fs.readFileSync(file, 'utf8') !== text) problems.push(`${name}: differs from freshly generated output`);
  }
  const sp = path.join(OUT, 'SOURCES.md');
  if (!fs.existsSync(sp) || fs.readFileSync(sp, 'utf8') !== sourcesText) problems.push('SOURCES.md: differs from freshly generated output');
  if (problems.length) { console.error('gen-vectors --check --online: ' + problems.length + ' problem(s)\n' + problems.join('\n')); process.exit(1); }
  console.log('gen-vectors --check --online: OK (byte for byte identical to upstream-derived output)');
} else {
  fs.mkdirSync(OUT, { recursive: true });
  for (const [name, text] of outputs) fs.writeFileSync(path.join(OUT, name), text);
  fs.writeFileSync(path.join(OUT, 'SOURCES.md'), sourcesText);
  console.log('SOURCES.md written');
}
