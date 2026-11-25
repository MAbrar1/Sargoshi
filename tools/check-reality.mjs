#!/usr/bin/env node
// "No fake, no placeholder" scanner. Exit code 1 on any violation.
// A line may opt out with:  SG-ALLOW: <specific reason>   (the reason is required and is listed in the report)
import fs from 'node:fs';
import path from 'node:path';
import { ROOT, walk, rel, isTestPath } from './lib.mjs';

const MOBILE = 'apps/mobile';
const files = ['apps', 'core', 'server', 'infra'].flatMap((d) => walk(path.join(ROOT, d), { exts: ['.dart', '.kt', '.rs', '.xml', '.kts', '.html', '.js', '.ts'] })).map(rel)
  .filter((f) => !/\.(g|freezed)\.dart$|(^|\/)generated\/|\.arb$|(^|\/)gen\//.test(f))
  .filter((f) => !f.startsWith('apps/mobile/ios/') && !f.includes('/.dart_tool/'));
const prod = files.filter((f) => !isTestPath(f));

const RULES = [
  { id: 'fake-class', re: /\b(Fake|Mock|Stub|Dummy|Noop|NoOp)[A-Z][A-Za-z0-9]*\b/, only: prod, msg: 'Fake/Mock/Stub/Dummy/Noop type in production code. Build the real thing. Test doubles belong in test directories only.' },
  { id: 'todo', re: /\b(TODO|FIXME|XXX|HACK)\b|lorem ipsum|coming soon|not (yet )?implemented|UnimplementedError|unimplemented!\s*\(|todo!\s*\(|\bPlaceholder\s*\(/i, only: prod, msg: 'Unfinished marker or placeholder in production code.' },
  { id: 'empty-callback', re: /\bon(Pressed|Tap|Changed|LongPress|DoubleTap|Submitted|Selected|Dismissed|Refresh|Toggle)\s*:\s*(\([^)]*\)|\w*)\s*(=>\s*\{\s*\}|\{\s*(\/\/[^\n]*)?\s*\})\s*[,)]/, only: prod.filter((f) => f.endsWith('.dart')), msg: 'Empty callback: every control must do something real.' },
  { id: 'null-handler', re: /\bon(Pressed|Tap|LongPress)\s*:\s*null\b/, only: prod.filter((f) => f.endsWith('.dart') && f.includes('/features/')), msg: 'Permanently disabled control (onPressed: null). Controls are enabled only by real state (use a named variable) or must not exist yet.' },
  { id: 'fake-delay', re: /Future\.delayed\(|Thread\.sleep\(|\bsleep\(Duration|tokio::time::sleep\(/, only: prod, msg: 'Delay in production code. Real work needs no artificial delay (use SG-ALLOW with a real timing reason such as backoff or debounce).' },
  { id: 'hex-colour', re: /#[0-9A-Fa-f]{6}\b|Color\(\s*0x[0-9A-Fa-f]{8}\s*\)/, only: prod.filter((f) => f.startsWith(MOBILE + '/lib/') && !f.includes('/core/theme/')), msg: 'Colour literal outside lib/core/theme. Use SargoshiColors tokens.' },
  { id: 'string-literal-ui', re: /\b(Text|SelectableText|AppBar)\(\s*(const\s+)?['"][^'"$]*[A-Za-z]{2,}[^'"]*['"]|\b(hintText|labelText|helperText|errorText|tooltip|semanticLabel|title|label)\s*:\s*(const\s+)?(Text\(\s*)?['"][^'"$]*[A-Za-z]{2,}[^'"]*['"]/, only: prod.filter((f) => f.startsWith(MOBILE + '/lib/') && !f.includes('/core/l10n/') && !f.includes('/debug/')), msg: 'User-visible string literal. Put it in app_en.arb and app_ur.arb and use the generated localisations.' },
  { id: 'print', re: /(^|[^\w.])(print|debugPrint)\(|\bprintln!\(|\bdbg!\(|\bLog\.[dviwe]\(|\bSystem\.out\.print/, only: prod.filter((f) => !f.includes('/bin/') && !/(^|\/)main\.rs$/.test(f)), msg: 'Direct print/log call. Use the redacting logger.' },
  { id: 'sample-data', re: /Ayesha|Bilal Ahmed|Abbu|Najjar|Bahawalpur|Zakat calculation/, only: prod.filter((f) => f.startsWith(MOBILE + '/lib/') || f.startsWith('server/') || f.startsWith('core/')), msg: 'Sample/figure data in production code. It belongs in test fixtures only.' },
  { id: 'unsafe-rust', re: /\bunsafe\s*(\{|fn|impl|trait)/, only: prod.filter((f) => f.endsWith('.rs') && !f.startsWith('core/crates/sg-ffi/')), msg: 'unsafe Rust outside sg-ffi.' },
  { id: 'ignore-lint', re: /\/\/\s*ignore(_for_file)?:\s*[a-z_,\s]+$|@Suppress\("(?!UNCHECKED_CAST)/, only: prod, msg: 'Lint suppression without a reason. Fix the code or add "SG-ALLOW: reason".' },
];

const violations = []; const allowed = [];
for (const f of files) {
  let txt; try { txt = fs.readFileSync(path.join(ROOT, f), 'utf8'); } catch { continue; }
  const lines = txt.split('\n');
  for (const rule of RULES) {
    if (!rule.only.includes(f)) continue;
    lines.forEach((l, i) => {
      if (!rule.re.test(l)) return;
      const m = (l + ' ' + (lines[i - 1] || '')).match(/SG-ALLOW:\s*(\S.{8,})/);
      if (m) allowed.push(`${f}:${i + 1} [${rule.id}] ${m[1].trim()}`);
      else violations.push(`${f}:${i + 1} [${rule.id}] ${rule.msg}\n    ${l.trim().slice(0, 140)}`);
    });
  }
}
// rust crates must forbid unsafe (except sg-ffi)
for (const f of walk(path.join(ROOT, 'core'), { exts: ['.rs'] }).concat(walk(path.join(ROOT, 'server'), { exts: ['.rs'] })).map(rel)) {
  if (/crates\/([^/]+)\/src\/lib\.rs$/.test(f) && !f.includes('sg-ffi')) {
    const t = fs.readFileSync(path.join(ROOT, f), 'utf8');
    if (!/#!\[forbid\(unsafe_code\)\]/.test(t)) violations.push(`${f}:1 [forbid-unsafe] crate root must contain #![forbid(unsafe_code)]`);
  }
}
// Android manifest hygiene
for (const m of walk(path.join(ROOT, MOBILE, 'android'), { exts: ['.xml'] }).map(rel).filter((f) => f.endsWith('main/AndroidManifest.xml'))) {
  const t = fs.readFileSync(path.join(ROOT, m), 'utf8');
  if (!/android:allowBackup="false"/.test(t)) violations.push(`${m}:1 [manifest] android:allowBackup must be "false" (Vault leak vector 4)`);
  if (!/android:dataExtractionRules=/.test(t)) violations.push(`${m}:1 [manifest] android:dataExtractionRules must exclude everything (Vault leak vector 4)`);
  if (/android:debuggable="true"/.test(t)) violations.push(`${m}:1 [manifest] debuggable must not be set in the main manifest`);
  if (/android:usesCleartextTraffic="true"/.test(t)) violations.push(`${m}:1 [manifest] cleartext traffic must not be enabled in the main manifest (use a debug-only network security config)`);
}
// secrets must not be committed
for (const f of walk(ROOT, { skip: ['docs'] }).map(rel)) {
  if (/(^|\/)(\.env|key\.properties|[^/]*\.jks|[^/]*\.keystore|service-account[^/]*\.json|google-services\.json)$/.test(f) && !/\.example$|\.template$/.test(f)) {
    try { const ig = fs.readFileSync(path.join(ROOT, '.gitignore'), 'utf8'); if (!ig.includes(path.basename(f)) && !ig.includes('*' + path.extname(f))) violations.push(`${f}:1 [secret] secret-like file is not git-ignored`); } catch { violations.push(`${f}:1 [secret] no .gitignore`); }
  }
}
if (allowed.length) { console.log(`SG-ALLOW exceptions (${allowed.length}):`); allowed.slice(0, 40).forEach((a) => console.log('  ' + a)); }
if (violations.length) {
  console.error(`check-reality: ${violations.length} violation(s)`);
  console.error(violations.slice(0, 80).join('\n') + (violations.length > 80 ? `\n... and ${violations.length - 80} more` : ''));
  process.exit(1);
}
console.log(`check-reality: OK (${files.length} source files scanned)`);
