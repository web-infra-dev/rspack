import fs from 'node:fs';
import path from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';

function write(file, content) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, content);
  const past = new Date(Date.now() - 60_000);
  fs.utimesSync(file, past, past);
}

async function waitFor(predicate) {
  const deadline = Date.now() + 5000;
  while (!predicate() && Date.now() < deadline) await sleep(10);
  expect(predicate()).toBe(true);
}

/** @type {import("@rspack/test-tools").TCompilerCaseConfig[]} */
export default [false, true].map((nativeWatcher) => ({
  name: nativeWatcher ? 'native' : 'watchpack',
  description:
    'should recover a pnpm workspace dependency after its dist is cleaned and rebuilt (#11851)',
  options(context) {
    const root = context.getDist(`symlink-recovery-${nativeWatcher}`);
    fs.rmSync(root, { recursive: true, force: true });
    const source = path.join(root, 'app');
    const library = path.join(root, 'lib');
    write(
      path.join(source, 'index.js'),
      'import value from "linked-library"; console.log(value);',
    );
    write(
      path.join(library, 'package.json'),
      '{"name":"linked-library","main":"dist/index.js"}',
    );
    write(path.join(library, 'dist/index.js'), 'export default "initial";');
    fs.mkdirSync(path.join(source, 'node_modules'));
    fs.symlinkSync(
      library,
      path.join(source, 'node_modules/linked-library'),
      'junction',
    );
    context.setValue('library', library);
    return {
      context: source,
      mode: 'development',
      devtool: false,
      entry: './index.js',
      output: { path: path.join(root, 'output') },
      experiments: { nativeWatcher },
    };
  },
  async build(context, compiler) {
    await sleep(2100);
    const library = context.getValue('library');
    const results = [];
    let fatalError;
    const watching = compiler.watch(
      { aggregateTimeout: 50, followSymlinks: true, ignored: [] },
      (error, stats) => {
        if (error) fatalError = error;
        else results.push(stats);
      },
    );
    try {
      await waitFor(() => results.some((stats) => !stats.hasErrors()));
      for (let cycle = 0; cycle < 2; cycle++) {
        await sleep(300);
        results.length = 0;
        fs.rmSync(path.join(library, 'dist'), { recursive: true });
        await waitFor(() => results.some((stats) => stats.hasErrors()));
        await sleep(300);
        results.length = 0;
        write(
          path.join(library, 'dist/index.js'),
          `export default "recovered-${cycle}";`,
        );
        await waitFor(() => results.some((stats) => !stats.hasErrors()));
        expect(fatalError).toBeUndefined();
        const output = results
          .at(-1)
          .compilation.assets['main.js'].source()
          .toString();
        expect(output).toContain(`recovered-${cycle}`);
      }
    } finally {
      await new Promise((resolve, reject) =>
        watching.close((error) => (error ? reject(error) : resolve())),
      );
    }
  },
}));
