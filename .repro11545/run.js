// End-to-end check for web-infra-dev/rspack#11545:
// with experiments.nativeWatcher on, ts-checker-rspack-plugin must observe file
// changes (via the new WatchFileSystem.on API) so typecheck results update.
const path = require('node:path');
const fs = require('node:fs');
const { rspack } = require('@rspack/core');
const { TsCheckerRspackPlugin } = require('ts-checker-rspack-plugin');

const SRC = path.join(__dirname, 'src/index.ts');
const ERROR_CONTENT = 'const variable: string = 123;\nexport { variable };\n';
const FIXED_CONTENT = 'const variable: number = 123;\nexport { variable };\n';

fs.writeFileSync(SRC, ERROR_CONTENT); // start from the broken state

const logs = [];
const passthroughErr = process.stderr.write.bind(process.stderr);
const passthroughOut = process.stdout.write.bind(process.stdout);
process.stderr.write = (c, ...a) => (
  logs.push(String(c)),
  passthroughErr(c, ...a)
);
process.stdout.write = (c, ...a) => (
  logs.push(String(c)),
  passthroughOut(c, ...a)
);

const drain = () => {
  const s = logs.join('');
  logs.length = 0;
  return s;
};
const hasTsError = (s) => /not assignable to type|TS2322/.test(s);
const sawChange = (s) => /Detected file change/.test(s);

const compiler = rspack({
  context: __dirname,
  mode: 'development',
  entry: './src/index.ts',
  experiments: { nativeWatcher: true },
  infrastructureLogging: { level: 'verbose', debug: true },
  resolve: { extensions: ['.ts', '.js'] },
  module: {
    rules: [
      {
        test: /\.ts$/,
        loader: 'builtin:swc-loader',
        options: { jsc: { parser: { syntax: 'typescript' } } },
      },
    ],
  },
  plugins: [new TsCheckerRspackPlugin()],
});

const watching = compiler.watch({ aggregateTimeout: 200 }, (err) => {
  if (err) console.error('[watch error]', err);
});

const wait = (ms) => new Promise((r) => setTimeout(r, ms));

(async () => {
  await wait(9000); // initial build + async typecheck (worker startup)
  const step0 = drain();
  const step0Error = hasTsError(step0);

  fs.writeFileSync(SRC, FIXED_CONTENT); // fix the error
  await wait(9000);
  const step1 = drain();

  const changeObserved = sawChange(step1);
  const errorClearedAfterFix = !hasTsError(step1);

  console.log('\n================ #11545 e2e result ================');
  console.log('nativeWatcher:                       on');
  console.log('step0 reported a type error:        ', step0Error);
  console.log('after edit, "Detected file change": ', changeObserved);
  console.log('after fix, type error cleared:      ', errorClearedAfterFix);
  const pass = step0Error && changeObserved && errorClearedAfterFix;
  console.log('RESULT:', pass ? 'PASS' : 'FAIL');
  console.log('==================================================\n');

  watching.close(() => process.exit(pass ? 0 : 1));
})();
