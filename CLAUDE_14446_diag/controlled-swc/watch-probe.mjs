// Same as ../controlled/watch-probe.mjs but the source is .tsx routed through
// builtin:swc-loader — to test whether the loader/build pipeline (not the
// dev-server/HMR) is what turns the recompiled module's dependency path into a
// mixed separator on Windows.
import { appendFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { rspack } from '@rspack/core';

const ctx = path.dirname(fileURLToPath(import.meta.url));
const editRel = 'src/App.tsx';
const editTarget = path.join(ctx, editRel);
const TIMEOUT_MS = 20000;

const cycles = [];
let timer;
let doneCount = 0;
const isSrc = (f) => /[\\/]src[\\/]/.test(f);

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
  console.log('\n==================== DIAGNOSIS (swc) ====================');
  console.log(
    `platform=${process.platform} node=${process.version} path.sep=${JSON.stringify(path.sep)}`,
  );
  cycles.forEach((c, i) => {
    console.log(`\n--- watcher INPUT, re-arm cycle #${i + 1} ---`);
    console.log(`  src files:`);
    console.log(fmtList(c.srcFiles));
  });
  console.log(`\nVERDICT: ${verdict}`);
  console.log('========================================================');
}

const compiler = rspack({
  context: ctx,
  mode: 'development',
  entry: { main: './src/main.tsx' },
  output: { path: path.join(ctx, 'dist') },
  resolve: { extensions: ['...', '.ts', '.tsx'] },
  module: {
    rules: [
      {
        test: /\.(jsx?|tsx?)$/,
        use: [
          {
            loader: 'builtin:swc-loader',
            options: { jsc: { parser: { syntax: 'typescript', tsx: true } } },
          },
        ],
        exclude: /node_modules/,
      },
    ],
  },
  infrastructureLogging: { level: 'error' },
  stats: 'errors-only',
});

const wfs = compiler.watchFileSystem;
const realWatch = wfs.watch.bind(wfs);
let cycleN = 0;
wfs.watch = function (files, dirs, missing, startTime, options, cb, cbu) {
  cycleN += 1;
  const fa = [...files];
  cycles.push({ srcFiles: fa.filter(isSrc) });
  const mixed = fa
    .filter(isSrc)
    .filter((f) => f.includes('/') && f.includes('\\'));
  console.log(
    `[watch re-arm #${cycleN}] files=${fa.length} mixedSepSrc=${JSON.stringify(mixed)}`,
  );
  return realWatch(files, dirs, missing, startTime, options, cb, cbu);
};

function arm() {
  clearTimeout(timer);
  timer = setTimeout(() => {
    dump(
      `WATCH STOPPED — no rebuild within ${TIMEOUT_MS}ms (#14446 REPRODUCED). builds=${doneCount}`,
    );
    process.exit(0);
  }, TIMEOUT_MS);
}

compiler.hooks.done.tap('probe', (stats) => {
  doneCount += 1;
  console.log(`[done #${doneCount}] hasErrors=${stats.hasErrors()}`);
  if (doneCount === 1 || doneCount === 2) {
    setTimeout(() => {
      console.log(`>> EDIT #${doneCount} -> ${editRel}`);
      appendFileSync(editTarget, `\n// edit${doneCount} ${Date.now()}\n`);
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
