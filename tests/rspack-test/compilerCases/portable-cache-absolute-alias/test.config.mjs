import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { createRequire } from 'node:module';
import { rspack } from '@rspack/core';
import {
  closeCompiler,
  runCompiler,
} from '@rspack/test-tools/helper/lifecycle';

const LOADER_RUNS = '__portableCacheAbsoluteAliasLoaderRuns';

async function build(
  context,
  cache = { type: 'persistent', portable: true },
  options = {},
  expectedErrors = [],
) {
  const loader = path.join(context, 'loader.cjs');
  if (!fs.existsSync(loader)) {
    fs.writeFileSync(
      loader,
      `module.exports = function(source, map) {
      globalThis[${JSON.stringify(LOADER_RUNS)}].push(this.resourcePath);
      this.callback(null, source, map);
    };`,
    );
  }
  globalThis[LOADER_RUNS] = [];
  const logs = [];
  const compiler = rspack({
    context,
    mode: 'development',
    target: 'node',
    devtool: false,
    entry: './src/index.js',
    output: {
      path: path.join(context, 'dist'),
      library: { type: 'commonjs2' },
      clean: true,
    },
    cache,
    module: {
      rules: [
        {
          test: /\.js$/,
          include: path.join(context, 'src'),
          loader: './loader.cjs',
        },
      ],
    },
    ...options,
    infrastructureLogging: {
      level: 'info',
      console: {
        ...console,
        info: (...args) => logs.push(args.join(' ')),
        log: () => {},
      },
    },
  });
  try {
    const stats = await runCompiler(compiler);
    const diagnostics = stats.toJson({
      all: false,
      errors: true,
      warnings: true,
    });
    expect(diagnostics.errors).toEqual(expectedErrors);
    const output = fs.readFileSync(
      path.join(context, 'dist', 'main.js'),
      'utf-8',
    );
    const values = [];
    const runtime = { exports: {} };
    const nodeRequire = createRequire(path.join(context, 'dist', 'main.js'));
    for (const filename of Object.keys(nodeRequire.cache)) {
      if (filename.startsWith(`${path.join(context, 'dist')}${path.sep}`))
        delete nodeRequire.cache[filename];
    }
    new Function('console', 'module', 'exports', 'require', output)(
      { log: (...args) => values.push(args.length === 1 ? args[0] : args) },
      runtime,
      runtime.exports,
      nodeRequire,
    );
    await (runtime.exports.default ?? runtime.exports);
    return {
      values,
      diagnostics,
      loaderRuns: globalThis[LOADER_RUNS].map((resource) =>
        path.relative(context, resource),
      ),
      assets: fs
        .readdirSync(path.join(context, 'dist'))
        .filter((file) => /\.(js|map)$/.test(file))
        .sort()
        .map((file) => [
          file,
          fs.readFileSync(path.join(context, 'dist', file), 'utf-8'),
        ]),
      built: [...stats.compilation.builtModules].map((module) =>
        path.relative(context, module.identifier()),
      ),
      dropped: logs.filter((line) =>
        /dropped \d+ entries bound to another root/.test(line),
      ),
      output,
    };
  } finally {
    await closeCompiler(compiler);
    delete globalThis[LOADER_RUNS];
  }
}

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
  description:
    'should validate fixed absolute resolver misses at their original location after a move',
  async run() {
    const temp = fs.realpathSync.native(
      fs.mkdtempSync(path.join(os.tmpdir(), 'rspack-portable-alias-')),
    );
    try {
      const writer = path.join(temp, 'a', 'proj');
      fs.mkdirSync(path.join(writer, '.git'), { recursive: true });
      fs.mkdirSync(path.join(writer, 'src'));
      fs.writeFileSync(
        path.join(writer, 'package.json'),
        '{ "name": "fixture" }',
      );
      fs.writeFileSync(path.join(writer, 'src', 'dep.js'), 'export default 7;');
      // An absolute alias fallback is fixed, even if it lies above the portable root.
      const fixed = path.join(temp, 'a', 'optional.js');
      fs.writeFileSync(
        path.join(writer, 'src', 'index.js'),
        'import v from "choice"; console.log(v);',
      );
      const options = { resolve: { alias: { choice: [fixed, './dep.js'] } } };
      await build(writer, undefined, options);
      const aliasReader = path.join(temp, 'alias', 'proj');
      fs.mkdirSync(path.dirname(aliasReader), { recursive: true });
      fs.cpSync(writer, aliasReader, { recursive: true });
      fs.writeFileSync(fixed, 'export default 99;');
      const aliasMoved = await build(aliasReader, undefined, options);
      const aliasCold = await build(aliasReader, false, options);
      expect(aliasMoved.output).toBe(aliasCold.output);
      expect(aliasMoved.values).toEqual([99]);
      expect(aliasCold.values).toEqual([99]);
      expect(aliasMoved.loaderRuns).not.toContain('src/index.js');
      const aliasRewarm = await build(aliasReader, undefined, options);
      expect(aliasRewarm.loaderRuns).toEqual([]);
      expect(aliasRewarm.output).toBe(aliasCold.output);

      // Keep the exact same typed configuration while moving the project. An absolute target
      // remains in A even if its spelling originally happened to be under the portable root.
      const fixedTarget = async (
        name,
        configure,
        relative = 'src/fixed.js',
      ) => {
        const origin = path.join(temp, name, 'a', 'proj');
        const destination = path.join(temp, name, 'b', 'proj');
        fs.mkdirSync(path.join(origin, '.git'), { recursive: true });
        fs.mkdirSync(path.join(origin, 'src'));
        fs.writeFileSync(
          path.join(origin, 'package.json'),
          '{ "name": "fixture" }',
        );
        const target = path.join(origin, relative);
        fs.mkdirSync(path.dirname(target), { recursive: true });
        fs.writeFileSync(target, 'export default 7;');
        const { request = 'choice', ...configuration } = configure(
          origin,
          target,
        );
        fs.writeFileSync(
          path.join(origin, 'src', 'index.js'),
          `import v from ${JSON.stringify(request)}; console.log(v);`,
        );
        await build(origin, undefined, configuration);
        const warm = await build(origin, undefined, configuration);
        expect(warm.built).toEqual([]);
        fs.mkdirSync(path.dirname(destination), { recursive: true });
        fs.cpSync(origin, destination, { recursive: true });
        fs.writeFileSync(
          path.join(destination, relative),
          'export default 99;',
        );
        const moved = await build(destination, undefined, configuration);
        const cold = await build(destination, false, configuration);
        expect(moved.values).toEqual([7]);
        expect(cold.values).toEqual([7]);
        expect(moved.output).toBe(cold.output);
        const rootBoundBuild =
          request.includes(origin) ||
          (process.platform !== 'win32' && path.isAbsolute(request));
        if (!rootBoundBuild) {
          expect(moved.loaderRuns).not.toContain('src/index.js');
        } else {
          // Absolute raw requests remain in build-owned NormalModule/dependency fields.
          // On POSIX, PortableString also binds /fixed.js as an outside-root path,
          // even though resolve.roots supplies its target. Keep that conservative
          // fallback; the relative-request cases above must still reuse their builds.
          expect(moved.loaderRuns).toContain('src/index.js');
        }
        const rewarm = await build(destination, undefined, configuration);
        expect(rewarm.loaderRuns).toEqual([]);
        expect(rewarm.output).toBe(cold.output);
      };
      await fixedTarget('inside', (_origin, target) => ({
        resolve: { alias: { choice: target } },
      }));
      await fixedTarget(
        'outside',
        (_origin, target) => ({
          resolve: { alias: { choice: target } },
        }),
        '../fixed.js',
      );
      await fixedTarget('fallback', (_origin, target) => ({
        resolve: { fallback: { choice: target } },
      }));
      await fixedTarget('specifier', (_origin, target) => ({
        request: target,
      }));
      await fixedTarget('modules', (_origin, target) => ({
        request: 'fixed.js',
        resolve: { modules: [path.dirname(target)] },
      }));
      await fixedTarget('roots', (_origin, target) => ({
        request: '/fixed.js',
        resolve: { roots: [path.dirname(target)] },
      }));
      await fixedTarget('tsconfig', (origin, target) => {
        const configFile = path.join(origin, 'tsconfig.json');
        fs.writeFileSync(
          configFile,
          JSON.stringify({
            compilerOptions: {
              baseUrl: '.',
              paths: { choice: [target] },
            },
          }),
        );
        return { resolve: { tsConfig: { configFile } } };
      });
      const envKey = 'RSPACK_PORTABLE_FIXED_ALIAS_TEST';
      const previous = process.env[envKey];
      try {
        await fixedTarget('environment', (_origin, target) => {
          process.env[envKey] = target;
          return { resolve: { alias: { choice: process.env[envKey] } } };
        });
      } finally {
        if (previous === undefined) delete process.env[envKey];
        else process.env[envKey] = previous;
      }

      // Exercise the repair with grouped imports, a separate async importer, maps, and a
      // second cache read. Both importers must keep their loader/parser output.
      const refreshResolution = async (
        name,
        configure,
        mutate,
        source,
        expected = [[99, 99], 99],
        expectedErrors = [],
      ) => {
        const origin = path.join(temp, name, 'a', 'proj');
        const destination = path.join(temp, name, 'b', 'proj');
        fs.mkdirSync(path.join(origin, '.git'), { recursive: true });
        fs.mkdirSync(path.join(origin, 'src'));
        fs.writeFileSync(
          path.join(origin, 'package.json'),
          '{"name":"fixture"}',
        );
        fs.writeFileSync(
          path.join(origin, 'src', 'first.js'),
          'export default 7;',
        );
        fs.writeFileSync(
          path.join(origin, 'src', 'second.js'),
          'export default 99;',
        );
        fs.writeFileSync(
          path.join(origin, 'src', 'lazy.js'),
          "import selected from 'choice'; export default selected;",
        );
        fs.writeFileSync(
          path.join(origin, 'src', 'index.js'),
          source ??
            `import selected from 'choice';
import * as grouped from 'choice';
console.log(selected, grouped.default);
export default import('./lazy.js').then(({default: value}) => console.log(value));`,
        );
        const configuration = { devtool: 'source-map', ...configure(origin) };
        await build(origin, undefined, configuration);
        expect(
          (await build(origin, undefined, configuration)).loaderRuns,
        ).toEqual([]);
        fs.mkdirSync(path.dirname(destination), { recursive: true });
        fs.cpSync(origin, destination, {
          recursive: true,
          verbatimSymlinks: true,
        });
        mutate(origin, destination);
        const moved = await build(
          destination,
          undefined,
          configuration,
          expectedErrors,
        );
        const cold = await build(
          destination,
          false,
          configuration,
          expectedErrors,
        );
        expect(moved.values).toEqual(expected);
        expect(moved.values).toEqual(cold.values);
        expect(moved.assets).toEqual(cold.assets);
        expect(moved.diagnostics).toEqual(cold.diagnostics);
        expect(moved.loaderRuns).not.toContain('src/index.js');
        expect(moved.loaderRuns).not.toContain('src/lazy.js');
        const rewarm = await build(
          destination,
          undefined,
          configuration,
          expectedErrors,
        );
        expect(rewarm.loaderRuns).toEqual([]);
        expect(rewarm.built).toEqual([]);
        expect(rewarm.values).toEqual(cold.values);
        expect(rewarm.assets).toEqual(cold.assets);
        expect(rewarm.diagnostics).toEqual(cold.diagnostics);
      };
      await refreshResolution(
        'grouped-alias',
        (origin) => ({
          resolve: { alias: { choice: path.join(origin, 'src', 'first.js') } },
        }),
        (origin) => {
          fs.writeFileSync(
            path.join(origin, 'src', 'first.js'),
            'export default 99;',
          );
        },
      );
      await refreshResolution(
        'mapping-change',
        (origin) => {
          const configFile = path.join(origin, 'tsconfig.json');
          fs.writeFileSync(
            configFile,
            JSON.stringify({
              compilerOptions: {
                baseUrl: '.',
                paths: { choice: ['./src/first.js'] },
              },
            }),
          );
          return { resolve: { tsConfig: { configFile } } };
        },
        (origin) => {
          fs.writeFileSync(
            path.join(origin, 'tsconfig.json'),
            JSON.stringify({
              compilerOptions: {
                baseUrl: '.',
                paths: { choice: ['./src/second.js'] },
              },
            }),
          );
        },
      );
      await refreshResolution(
        'resolution-diagnostics',
        (origin) => ({
          resolve: { alias: { choice: path.join(origin, 'src', 'first.js') } },
        }),
        (origin) => fs.unlinkSync(path.join(origin, 'src', 'first.js')),
        `let value; try { value = require('choice').default; } catch { value = 'missing'; } console.log(value);`,
        ['missing'],
        // AliasNotFound is an error even for an optional require in a cold build. Check
        // that exact diagnostic, including reader locations, without rebuilding the importer.
        [
          expect.objectContaining({
            message: expect.stringContaining(
              "Cannot find module 'choice' for matched aliased key 'choice'",
            ),
          }),
        ],
      );

      if (process.platform !== 'win32') {
        for (const absolute of [false, true]) {
          await refreshResolution(
            `retarget-${absolute ? 'absolute' : 'relative'}`,
            (origin) => {
              fs.mkdirSync(path.join(origin, 'node_modules'));
              for (const [name, value] of [
                ['first', 7],
                ['second', 99],
              ]) {
                const directory = path.join(origin, 'packages', name);
                fs.mkdirSync(directory, { recursive: true });
                fs.writeFileSync(
                  path.join(directory, 'index.js'),
                  `export default ${value};`,
                );
              }
              fs.symlinkSync(
                absolute
                  ? path.join(origin, 'packages', 'first')
                  : '../packages/first',
                path.join(origin, 'node_modules', 'choice'),
                'dir',
              );
              return {};
            },
            (origin, destination) => {
              const link = path.join(destination, 'node_modules', 'choice');
              fs.unlinkSync(link);
              fs.symlinkSync(
                absolute
                  ? path.join(origin, 'packages', 'second')
                  : '../packages/second',
                link,
                'dir',
              );
            },
          );
        }
      }
      // An unused absolute replacement supplies no fixed resolution observation. Its presence
      // in configuration must not prevent ordinary ancestor lookups from being reused.
      const unusedWriter = path.join(temp, 'unused', 'a', 'proj');
      const unusedReader = path.join(temp, 'unused', 'b', 'proj');
      fs.mkdirSync(path.join(unusedWriter, '.git'), { recursive: true });
      fs.mkdirSync(path.join(unusedWriter, 'src'));
      fs.writeFileSync(
        path.join(unusedWriter, 'src', 'index.js'),
        'console.log(7);',
      );
      const unusedOptions = { resolve: { alias: { unused: fixed } } };
      await build(unusedWriter, undefined, unusedOptions);
      fs.mkdirSync(path.dirname(unusedReader), { recursive: true });
      fs.cpSync(unusedWriter, unusedReader, { recursive: true });
      const unusedMoved = await build(unusedReader, undefined, unusedOptions);
      expect(unusedMoved.built).toEqual([]);
      const unusedCold = await build(unusedReader, false, unusedOptions);
      expect(unusedMoved.output).toBe(unusedCold.output);

      if (process.platform !== 'win32') {
        await fixedTarget(
          'symlink',
          (origin, target) => {
            fs.mkdirSync(path.join(origin, 'node_modules'));
            fs.symlinkSync(
              path.dirname(target),
              path.join(origin, 'node_modules', 'choice'),
              'dir',
            );
            return {};
          },
          'target/index.js',
        );
      }
    } finally {
      fs.rmSync(temp, { recursive: true, force: true });
    }
  },
};
