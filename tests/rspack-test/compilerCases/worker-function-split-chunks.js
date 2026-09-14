const path = require('node:path');
const { rspack, workerFunction } = require('@rspack/core');
const { createFsFromVolume, Volume } = require('memfs');
const fixture = path.join(__dirname, 'fixtures/worker-function-split-chunks');
const run = compiler => new Promise((resolve, reject) => compiler.run((error, stats) => error ? reject(error) : resolve(stats)));
const close = compiler => new Promise((resolve, reject) => compiler.close(error => error ? reject(error) : resolve()));
function create(name, group = false, target = 'name.cjs') {
  const compiler = rspack({
    context: fixture, mode: 'development', devtool: false, cache: false, entry: './entry.js',
    output: { path: '/out' },
    resolveLoader: { alias: { naming: path.join(fixture, target) } },
    optimization: {
      minimize: false, concatenateModules: false,
      splitChunks: {
        chunks: 'all', minSize: 0, ...(group ? {} : { name }),
        cacheGroups: { default: false, defaultVendors: false, shared: {
          test: /value\.js$/, enforce: true, ...(group ? { name } : {}),
        } },
      },
    },
  });
  compiler.outputFileSystem = createFsFromVolume(new Volume());
  return compiler;
}
module.exports = [
  ...[false, true].flatMap(group => ['name.cjs', 'name.mjs'].map(target => ({
    description: `workerFunction splitChunks.name supports ${group ? 'cache groups' : 'top level'} with ${target}, nested options and rebuilds`,
    async build() {
      const name = workerFunction('naming', {
        name: 'worker', transform: workerFunction('./transform.mjs', { suffix: '-shared' }),
      });
      const compilers = [create(name, group, target), create(name, group, target)];
      try {
        for (let build = 0; build < 2; build++) {
          const results = await Promise.all(compilers.map(run));
          for (const stats of results) {
            expect(stats.hasErrors()).toBe(false);
            const chunk = stats.toJson({ all: false, modules: true, cachedModules: true, chunks: true, chunkModules: true, dependentModules: true, chunkModulesSpace: Infinity }).chunks.find(chunk => chunk.names.includes('worker-shared'));
            expect(chunk.modules.map(module => module.name)).toContain('./value.js');
          }
        }
      } finally { await Promise.all(compilers.map(close)); }
    },
  }))),
  {
    description: 'workerFunction splitChunks.name supports undefined names',
    async build() {
      const compiler = create(workerFunction('naming', { kind: 'undefined' }));
      try { expect((await run(compiler)).hasErrors()).toBe(false); }
      finally { await close(compiler); }
    },
  },
  ...[
    ['throw', /split name worker failure/], ['crash', /worker dropped/],
    ['invalid', /must return a string or undefined/], ['null', /must return a string or undefined/],
    ['export', /must export a function/], ['missing', /Cannot resolve workerFunction/],
  ].map(([kind, expected]) => ({
    description: `workerFunction splitChunks.name ${kind} reports an error and permits a subsequent worker build`,
    async build() {
      let compiler;
      try {
        let message;
        try {
          compiler = create(workerFunction('naming', { kind }), false,
            kind === 'export' ? 'not-function.cjs' : kind === 'missing' ? 'missing.cjs' : 'name.cjs');
          message = (await run(compiler)).toString({ all: false, errors: true });
        }
        catch (error) { message = error.message; }
        expect(message).toMatch(expected);
      } finally { if (compiler) await close(compiler); }
      const next = create(workerFunction('naming', { name: 'recovered' }));
      try {
        const stats = await run(next);
        expect(stats.hasErrors()).toBe(false);
        expect(stats.toJson({ all: false, assets: true }).assets.some(asset => asset.name === 'recovered.js')).toBe(true);
      } finally { await close(next); }
    },
  })),
];
