import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { rspack } from '@rspack/core';
import {
  closeCompiler,
  runCompiler,
} from '@rspack/test-tools/helper/lifecycle';

const LOADER_RUNS = '__portableCacheDroppedTargetLoaderRuns';

async function build(context, cache = { type: 'persistent', portable: true }) {
  globalThis[LOADER_RUNS] = [];
  const compiler = rspack({
    context,
    mode: 'development',
    devtool: false,
    entry: './src/index.js',
    output: { path: path.join(context, 'dist') },
    module: {
      rules: [
        {
          test: /\.js$/,
          include: path.join(context, 'src'),
          loader: './loader.cjs',
        },
      ],
    },
    cache,

  });
  try {
    const stats = await runCompiler(compiler);
    expect(stats.toJson({ all: false, errors: true }).errors).toEqual([]);
    const { logging } = stats.toJson({ all: false, logging: false, loggingDebug: /^rspack\.persistentCache$/ });
    const logs = (logging?.['rspack.persistentCache']?.entries ?? []).map(entry => entry.message);
    const dist = path.join(context, 'dist');
    return {
      built: [...stats.compilation.builtModules]
        .map((module) => path.relative(context, module.resource).split(path.sep).join('/'))
        .sort(),
      loaderRuns: globalThis[LOADER_RUNS]
        .map((resource) => path.relative(context, resource).split(path.sep).join('/'))
        .sort(),
      dropped: logs.filter((line) =>
        /^make persistent cache dropped \d+ entries bound to another root/.test(
          line,
        ),
      ),
      output: fs
        .readdirSync(dist)
        .filter((file) => file.endsWith('.js'))
        .sort()
        .map(
          (file) => `${file}\n${fs.readFileSync(path.join(dist, file), 'utf-8')}`,
        )
        .join('\n'),
    };
  } finally {
    delete globalThis[LOADER_RUNS];
    await closeCompiler(compiler);
  }
}

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
  description:
    'should refactorize only the dependencies whose root-bound targets were dropped after a move',
  async run() {
    const temp = fs.realpathSync.native(
      fs.mkdtempSync(path.join(os.tmpdir(), 'rspack-portable-dropped-target-')),
    );
    try {
      const writer = path.join(temp, 'a', 'proj');
      fs.mkdirSync(path.join(writer, '.git'), { recursive: true });
      fs.mkdirSync(path.join(writer, 'src'));
      fs.writeFileSync(
        path.join(writer, 'loader.cjs'),
        `module.exports = function (source) {
  globalThis[${JSON.stringify(LOADER_RUNS)}].push(this.resourcePath);
  return source;
};
`,
      );
      // `import.meta.url` binds `child.js` and `lazy-child.js` to the writer's root, so a moved
      // cache drops them. `index.js` and `stable.js` stay portable.
      fs.writeFileSync(
        path.join(writer, 'src', 'index.js'),
        `import stable from './stable.js';
import child from './child.js';
import * as childNamespace from './child.js';
console.log(stable, child, childNamespace.default);
import('./lazy-child.js').then(({ default: lazy }) => console.log(lazy));
export default 'index';
`,
      );
      fs.writeFileSync(
        path.join(writer, 'src', 'stable.js'),
        "export default 'stable';\n",
      );
      // The cycle back to the entry and the shared `stable.js` must reconnect to recovered modules.
      fs.writeFileSync(
        path.join(writer, 'src', 'child.js'),
        `import index from './index.js';
import stable from './stable.js';
export default [import.meta.url, stable, () => index];
`,
      );
      fs.writeFileSync(
        path.join(writer, 'src', 'lazy-child.js'),
        `import stable from './stable.js';
export default [import.meta.url, stable];
`,
      );
      await build(writer);
      const warm = await build(writer);
      expect(warm.built).toEqual([]);
      expect(warm.loaderRuns).toEqual([]);

      // A sibling directory keeps the layout above the root, like another git worktree.
      const reader = path.join(temp, 'b', 'proj');
      fs.mkdirSync(path.dirname(reader), { recursive: true });
      fs.cpSync(writer, reader, { recursive: true });
      const moved = await build(reader);
      expect(moved.dropped).toEqual([
        'make persistent cache dropped 2 entries bound to another root',
      ]);
      // Only the dropped targets rebuild. Their importers keep the cached build.
      expect(moved.built).toEqual(['src/child.js', 'src/lazy-child.js']);
      expect(moved.loaderRuns).toEqual(['src/child.js', 'src/lazy-child.js']);
      const cold = await build(reader, false);
      expect(cold.loaderRuns).toEqual([
        'src/child.js',
        'src/index.js',
        'src/lazy-child.js',
        'src/stable.js',
      ]);
      expect(moved.output).toBe(cold.output);
      expect(moved.output).toContain(
        pathToFileURL(path.join(reader, 'src', 'child.js')).href,
      );

      // The cache written after the move recovers completely.
      const rewarm = await build(reader);
      expect(rewarm.dropped).toEqual([]);
      expect(rewarm.built).toEqual([]);
      expect(rewarm.loaderRuns).toEqual([]);
      expect(rewarm.output).toBe(cold.output);
    } finally {
      fs.rmSync(temp, { recursive: true, force: true });
    }
  },
};
