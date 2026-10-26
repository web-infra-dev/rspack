import fs from 'node:fs';
import path from 'node:path';

const CASE_NAME = 'native-watcher-removed-file-once';
const AGGREGATE_TIMEOUT = 200;
const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
  description:
    'should rebuild once when a removed file stays a file dependency',
  options(context) {
    const source = path.join(context.getDist(CASE_NAME), 'src');
    fs.rmSync(context.getDist(CASE_NAME), { recursive: true, force: true });
    fs.mkdirSync(source, { recursive: true });
    const entry = path.join(source, 'index.js');
    const watched = path.join(source, 'watched.txt');
    fs.writeFileSync(entry, 'module.exports = 1;\n');
    fs.writeFileSync(watched, 'watched\n');
    // Keep initial writes outside the first build's mtime accuracy window.
    const past = new Date(Date.now() - 10_000);
    for (const file of [entry, watched]) fs.utimesSync(file, past, past);
    context.setValue('watched', watched);
    return {
      context: source,
      mode: 'development',
      devtool: false,
      target: 'node',
      entry: './index.js',
      output: { path: path.join(context.getDist(CASE_NAME), 'dist') },
      experiments: { nativeWatcher: true },
      plugins: [
        (compiler) => {
          compiler.hooks.thisCompilation.tap(CASE_NAME, (compilation) => {
            compilation.fileDependencies.add(watched);
          });
        },
      ],
    };
  },
  async build(context, compiler) {
    let builds = 0;
    let failure;
    let watching;
    const firstBuild = new Promise((resolve, reject) => {
      watching = compiler.watch(
        { aggregateTimeout: AGGREGATE_TIMEOUT },
        (error, stats) => {
          if (error || stats.hasErrors()) {
            failure = error || new Error(stats.toString());
            reject(failure);
            return;
          }
          builds++;
          resolve();
        },
      );
    });
    try {
      await firstBuild;
      await wait(AGGREGATE_TIMEOUT * 2);
      fs.unlinkSync(context.getValue('watched'));
      await wait(AGGREGATE_TIMEOUT * 6);
      if (failure) throw failure;
      context.setValue('builds', builds);
    } finally {
      await new Promise((resolve, reject) =>
        watching.close((error) => (error ? reject(error) : resolve())),
      );
    }
  },
  async check({ context }) {
    expect(context.getValue('builds')).toBe(2);
  },
};
