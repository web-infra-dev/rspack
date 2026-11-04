// Run the copied harness in .bench/js-module-bindings; see module-bindings.md.
(async () => {
  const assert = require('node:assert/strict');
  const { createRequire } = require('node:module');
  const path = require('node:path');
  const { pathToFileURL } = require('node:url');
  const fs = require('node:fs');
  const { Bench } = await import('tinybench');
  const { withCodSpeed } = await import('@codspeed/tinybench-plugin');

  const root = path.resolve(__dirname, '../..');
  const testRequire = createRequire(
    path.join(root, 'tests/rspack-test/package.json'),
  );
  const { Volume, createFsFromVolume } = testRequire('memfs');
  const core = await import(
    pathToFileURL(
      process.env.MODULE_BENCH_CORE ||
        path.join(root, 'packages/rspack/dist/index.js'),
    )
  );
  const { rspack } = core;
  const count = Number(process.env.MODULE_BENCH_COUNT || 2000);
  const loader = path.join(__dirname, 'identity-loader.cjs');
  fs.writeFileSync(
    loader,
    'module.exports = function(source) { return source; };\n',
  );
  const files = {
    '/project/index.js': Array.from(
      { length: count },
      (_, i) => `import './m${i}.js';`,
    ).join('\n'),
  };
  for (let i = 0; i < count; i++)
    files[`/project/m${i}.js`] = `globalThis.v${i} = ${i};`;
  const input = createFsFromVolume(Volume.fromJSON(files));
  input.join = path.join;
  let checksum = 0;
  const makeBench = () =>
    withCodSpeed(
      new Bench({
        time: 2000,
        iterations: 20,
        warmupTime: 1000,
        warmupIterations: 5,
      }),
    );
  const selected = process.env.MODULE_BENCH_FILTER || '';

  function compilerFor(access, mode = 'development') {
    const compiler = rspack({
      context: '/project',
      experiments: { useInputFileSystem: [/^\/project/] },
      mode,
      entry: './index.js',
      cache: false,
      devtool: false,
      output: { path: '/output', filename: 'main.js' },
      optimization: { minimize: false, concatenateModules: false },
      module:
        access === 'js-loader'
          ? { rules: [{ test: /m\d+\.js$/, use: loader }] }
          : undefined,
      plugins:
        access && access !== 'js-loader'
          ? [
              (compiler) =>
                compiler.hooks.compilation.tap('Bench', (compilation) => {
                  compilation.hooks.finishModules.tap('Bench', (modules) => {
                    let seen = 0;
                    for (const module of modules) {
                      seen++;
                      checksum += module.identifier().length;
                      if (access === 'all-fields') {
                        checksum +=
                          module.type.length +
                          module.context.length +
                          module.resource.length;
                        checksum +=
                          module.request.length +
                          module.userRequest.length +
                          module.rawRequest.length;
                        checksum +=
                          module.loaders.length +
                          module.dependencies.length +
                          module.blocks.length;
                        checksum +=
                          module.resourceResolveData.path.length +
                          module.readableIdentifier().length;
                        checksum +=
                          module.size() + module.nameForCondition().length;
                        checksum +=
                          Number(module.useSourceMap) +
                          Number(module.useSimpleSourceMap);
                        assert.equal(module.error, undefined);
                        assert.equal(module.buildInfo, module.buildInfo);
                      }
                    }
                    assert.equal(seen, count + 1);
                  });
                }),
            ]
          : [],
    });
    compiler.inputFileSystem = input;
    compiler.outputFileSystem = createFsFromVolume(new Volume());
    return compiler;
  }
  function run(compiler) {
    return new Promise((resolve, reject) =>
      compiler.run((error, stats) => {
        if (error) reject(error);
        else if (stats.hasErrors())
          reject(new Error(stats.toString({ all: false, errors: true })));
        else resolve(stats);
      }),
    );
  }
  const close = (compiler) =>
    new Promise((resolve, reject) =>
      compiler.close((error) => (error ? reject(error) : resolve())),
    );
  const builds = makeBench();
  for (const mode of ['development', 'production']) {
    for (const access of ['', 'identifier', 'all-fields', 'js-loader']) {
      const name = `build/${mode}/${access || 'no-module-api'}`;
      if (!name.includes(selected)) continue;
      builds.add(name, async () => {
        const compiler = compilerFor(access, mode);
        try {
          await run(compiler);
        } finally {
          await close(compiler);
        }
      });
    }
  }
  for (const task of builds.tasks)
    task.addEventListener('error', () => console.error(task.result?.error));
  if (builds.tasks.length) await builds.run();
  if (!selected || 'api'.includes(selected)) {
    const compiler = compilerFor('');
    compiler.hooks.compilation.tap('ApiBench', (compilation) => {
      compilation.hooks.finishModules.tapPromise(
        'ApiBench',
        async (modules) => {
          const selectedModules = [...modules]
            .filter((module) => /m\d+\.js$/.test(module.resource))
            .slice(0, 64);
          const module = selectedModules[0];
          const bench = makeBench();
          const repeats = 10000;
          bench.add('api/snapshot-fields', () => {
            for (let i = 0; i < repeats; i++) {
              const current = selectedModules[i % selectedModules.length];
              checksum +=
                current.resource.length +
                current.request.length +
                current.type.length +
                current.identifier().length;
            }
          });
          bench.add('api/live-getters', () => {
            for (let i = 0; i < repeats; i++)
              checksum += module.context.length + Number(module.useSourceMap);
          });
          bench.add('api/methods', () => {
            for (let i = 0; i < repeats; i++)
              checksum += module.size() + module.nameForCondition().length;
          });
          await bench.run();
        },
      );
    });
    try {
      await run(compiler);
    } finally {
      await close(compiler);
    }
  }
  assert.ok(Number.isFinite(checksum));
})().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
