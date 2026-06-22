// Empirical probe: writes real files, watches them with the *real* watchpack
// (the same version rspack depends on), then reads the per-entry `accuracy`
// that watchpack's `ensureFsAccuracy` ladder converged to on this OS.
//
// Note: watchpack feeds `+stats.mtime` (integer ms) into the ladder, so its
// `% 1` branch never fires in practice — the effective floor is 10ms, matching
// the native (Rust) watcher.
import Watchpack from 'watchpack';
import { mkdtempSync, writeFileSync, statSync, appendFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';

const COUNT = 50;

const dir = mkdtempSync(join(tmpdir(), 'fs-accuracy-'));
const files = [];

// Spread files over ~1.1s of wall clock so their mtimes reveal the filesystem's
// real granularity.
for (let i = 0; i < COUNT; i++) {
  const file = join(dir, `probe_${i}.txt`);
  writeFileSync(file, String(i));
  files.push(file);
  await sleep(21 + ((i * 271) % 900) / 1000);
}

const mtimes = files.map((f) => +statSync(f).mtime);

const wp = new Watchpack({ aggregateTimeout: 100, poll: false });

const aggregated = new Promise((resolve) => wp.once('aggregated', resolve));
wp.watch({ files, startTime: Date.now() - 10_000 });

// Touch every file once so watchpack rescans and populates time-info entries.
await sleep(200);
for (const f of files) appendFileSync(f, '\n');

await Promise.race([aggregated, sleep(3000)]);

const entries = wp.getTimeInfoEntries();
const accuracies = [];
for (const f of files) {
  const entry = entries.get(f);
  if (entry && typeof entry.accuracy === 'number')
    accuracies.push(entry.accuracy);
}
wp.close();

const converged = accuracies.length ? Math.min(...accuracies) : null;

const sorted = [...new Set(mtimes)].sort((a, b) => a - b);
let minDelta = 0;
for (let i = 1; i < sorted.length; i++) {
  const d = sorted[i] - sorted[i - 1];
  if (d > 0 && (minDelta === 0 || d < minDelta)) minDelta = d;
}
const nonMult10 = mtimes.filter((m) => m % 10 !== 0).length;
const nonMult100 = mtimes.filter((m) => m % 100 !== 0).length;
const nonMult1000 = mtimes.filter((m) => m % 1000 !== 0).length;

console.log(`FS_ACCURACY_PROBE watchpack os=${process.platform}`);
console.log(`FS_ACCURACY_PROBE watchpack converged_accuracy_ms=${converged}`);
console.log(
  `FS_ACCURACY_PROBE watchpack empirical_min_mtime_delta_ms=${minDelta}`,
);
console.log(
  `FS_ACCURACY_PROBE watchpack samples=${COUNT} distinct=${sorted.length} entries_with_accuracy=${accuracies.length} non_mult_10=${nonMult10} non_mult_100=${nonMult100} non_mult_1000=${nonMult1000}`,
);

process.exit(0);
