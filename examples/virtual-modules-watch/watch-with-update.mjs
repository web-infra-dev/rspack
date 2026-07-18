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
  output: { path: path.resolve(__dirname, 'dist-update') },
  experiments: { nativeWatcher: true, inputFileSystem: [/src\/virtual.*\.js/] },
  plugins: [virtualModules],
  infrastructureLogging: { level: 'verbose' },
});

let compilationCount = 0;
compiler.hooks.done.tap('repro', (stats) => {
  compilationCount += 1;
  console.log(
    `[compile #${compilationCount}] hash=${stats.hash} modified=${JSON.stringify(
      [...(stats.compilation.modifiedFiles ?? [])],
    )}`,
  );
});

const watching = compiler.watch({ aggregateTimeout: 50 }, (err) => {
  if (err) {
    console.error('watch error:', err);
    process.exit(1);
  }
});

setTimeout(() => {
  console.log('-> writing new virtual content');
  virtualModules.writeModule(
    VIRTUAL_ENTRY,
    'export const payload = "updated";',
  );
}, 800);

setTimeout(() => {
  watching.close(() => {
    compiler.close(() => {
      console.log(`\ntotal compilations: ${compilationCount}`);
      if (compilationCount === 2) {
        console.log('[OK] exactly 1 initial + 1 update');
      } else {
        console.log(`[WARN] expected 2, got ${compilationCount}`);
        process.exitCode = 2;
      }
    });
  });
}, 2500);
