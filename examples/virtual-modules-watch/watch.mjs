import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { rspack } from '@rspack/core';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const VIRTUAL_ENTRY = path.resolve(__dirname, 'src/virtual-payload.js');

const virtualModules = new rspack.experiments.VirtualModulesPlugin({
  [VIRTUAL_ENTRY]: 'export const payload = "initial";',
});

const compiler = rspack({
  context: __dirname,
  mode: 'development',
  entry: './src/index.js',
  output: { path: path.resolve(__dirname, 'dist') },
  experiments: { nativeWatcher: true, inputFileSystem: [/src\/virtual.*\.js/] },
  plugins: [virtualModules],
  infrastructureLogging: { level: 'error' },
});

let compilationCount = 0;
const events = [];

compiler.hooks.done.tap('repro', (stats) => {
  compilationCount += 1;
  const { compilation } = stats;
  events.push({
    n: compilationCount,
    t: Date.now(),
    hash: stats.hash,
    changedFiles: [...(compilation.watchFiles ?? [])].filter((f) =>
      stats.compilation.fileDependencies.has(f),
    ).length,
    modified: [...(compilation.modifiedFiles ?? [])],
    removed: [...(compilation.removedFiles ?? [])],
  });
  console.log(
    `[compile #${compilationCount}] hash=${stats.hash} modified=${JSON.stringify(
      [...(compilation.modifiedFiles ?? [])],
    )} removed=${JSON.stringify([...(compilation.removedFiles ?? [])])}`,
  );
});

const watching = compiler.watch({ aggregateTimeout: 50 }, (err) => {
  if (err) {
    console.error('watch error:', err);
    process.exit(1);
  }
});

setTimeout(() => {
  watching.close(() => {
    compiler.close(() => {
      console.log('\n=== SUMMARY ===');
      console.log(`total compilations: ${compilationCount}`);
      console.log('events:', JSON.stringify(events, null, 2));
      if (compilationCount > 1) {
        console.log(
          `\n[REPRO] redundant rebuild detected (expected 1, saw ${compilationCount})`,
        );
        process.exitCode = 2;
      } else {
        console.log('\n[OK] no redundant rebuild');
      }
    });
  });
}, 3000);
