// Diagnostic probe for web-infra-dev/rspack#14446
// "File watching stops after first change on Windows".
//
// Drives compiler.watch(), edits a source file between builds, and records:
//   1. whether each edit actually triggers a rebuild (the user-visible symptom)
//   2. the EXACT file/dir path strings handed to the watcher on every re-arm
//      cycle (the diagnosis: do they change shape between cycle #1 and #2?)
//
// Exit code is always 0; the verdict is printed as a `VERDICT:` line that the
// workflow parses. The harness is identical on every OS, so a divergence in the
// result across windows/ubuntu/macos isolates the platform-specific cause.

import { appendFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { rspack } from '@rspack/core';

const ctx = path.dirname(fileURLToPath(import.meta.url));
const editRel = 'src/App.js';
const editTarget = path.join(ctx, editRel);
const TIMEOUT_MS = 20000;

const cycles = [];
let timer;
let doneCount = 0;

const isSrc = (f) => /[\\/]src[\\/]/.test(f);
const isSrcDir = (f) => /[\\/]src([\\/]|$)/.test(f);

function fmtList(list) {
  if (list.length === 0) return '      (none)';
  return list
    .slice()
    .sort()
    .map(
      (f) =>
        `      ${JSON.stringify(f)}  normalized=${path.normalize(f) === f}`,
    )
    .join('\n');
}

function dump(verdict) {
  console.log('\n==================== DIAGNOSIS ====================');
  console.log(
    `platform=${process.platform} node=${process.version} path.sep=${JSON.stringify(path.sep)}`,
  );
  console.log(`edit target string fed by test = ${JSON.stringify(editTarget)}`);
  cycles.forEach((c, i) => {
    console.log(
      `\n--- watcher INPUT, re-arm cycle #${i + 1} (startTime=${c.startTime}) ---`,
    );
    console.log(`  files: total=${c.filesTotal}, under src:`);
    console.log(fmtList(c.srcFiles));
    console.log(`  directories: total=${c.dirsTotal}, under src:`);
    console.log(fmtList(c.srcDirs));
    console.log(`  App.js present in files set: ${c.hasApp}`);
  });
  if (cycles.length >= 2) {
    const s1 = new Set(cycles[0].srcFiles);
    const s2 = new Set(cycles[1].srcFiles);
    const d1 = new Set(cycles[0].srcDirs);
    const d2 = new Set(cycles[1].srcDirs);
    console.log(
      '\n--- DIFF of src watch-input strings: cycle #1 -> cycle #2 ---',
    );
    console.log(
      '  files only in cycle #1:',
      [...s1].filter((x) => !s2.has(x)),
    );
    console.log(
      '  files only in cycle #2:',
      [...s2].filter((x) => !s1.has(x)),
    );
    console.log(
      '  dirs  only in cycle #1:',
      [...d1].filter((x) => !d2.has(x)),
    );
    console.log(
      '  dirs  only in cycle #2:',
      [...d2].filter((x) => !d1.has(x)),
    );
  }
  console.log(`\nVERDICT: ${verdict}`);
  console.log('==================================================');
}

const compiler = rspack({
  context: ctx,
  mode: 'development',
  entry: { main: './src/main.js' },
  output: { path: path.join(ctx, 'dist') },
  infrastructureLogging: { level: 'error' },
  stats: 'errors-only',
});

// Capture exactly what is handed to the watcher on each re-arm cycle.
const wfs = compiler.watchFileSystem;
console.log(`watchFileSystem = ${wfs?.constructor?.name}`);
const realWatch = wfs.watch.bind(wfs);
let cycleN = 0;
wfs.watch = function (files, dirs, missing, startTime, options, cb, cbu) {
  cycleN += 1;
  const fa = [...files];
  const da = [...dirs];
  cycles.push({
    startTime,
    filesTotal: fa.length,
    dirsTotal: da.length,
    srcFiles: fa.filter(isSrc),
    srcDirs: da.filter(isSrcDir),
    hasApp: fa.some((f) => f.endsWith(`App.js`)),
  });
  console.log(
    `[watch re-arm #${cycleN}] files=${fa.length} dirs=${da.length} startTime=${startTime}`,
  );
  return realWatch(files, dirs, missing, startTime, options, cb, cbu);
};

function arm() {
  clearTimeout(timer);
  timer = setTimeout(() => {
    dump(
      `WATCH STOPPED — no rebuild within ${TIMEOUT_MS}ms after an edit (#14446 REPRODUCED). builds_seen=${doneCount}`,
    );
    process.exit(0);
  }, TIMEOUT_MS);
}

compiler.hooks.done.tap('probe', (stats) => {
  doneCount += 1;
  console.log(
    `[done #${doneCount}] rebuild completed (hasErrors=${stats.hasErrors()})`,
  );
  if (doneCount === 1) {
    setTimeout(() => {
      console.log(`>> EDIT #1 -> ${editRel}`);
      appendFileSync(editTarget, `\n// edit1 ${Date.now()}\n`);
      arm();
    }, 1200);
  } else if (doneCount === 2) {
    setTimeout(() => {
      console.log(`>> EDIT #2 -> ${editRel}`);
      appendFileSync(editTarget, `\n// edit2 ${Date.now()}\n`);
      arm();
    }, 1200);
  } else {
    clearTimeout(timer);
    dump(
      'WATCH OK — both edits triggered rebuilds (#14446 NOT reproduced here)',
    );
    process.exit(0);
  }
});

compiler.watch({}, (err) => {
  if (err) {
    console.error('watch callback error:', err);
    process.exit(1);
  }
});
