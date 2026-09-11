const path = require('node:path');
const { rspack, workerFunction } = require('@rspack/core');
const fixture = path.join(__dirname, 'fixtures/worker-function');
const run = compiler => new Promise((resolve, reject) => compiler.run((error, stats) => error ? reject(error) : resolve(stats)));
const close = compiler => new Promise((resolve, reject) => compiler.close(error => error ? reject(error) : resolve()));

module.exports = [{
  description: 'workerFunction rejects unsupported registrations and interceptor combinations',
  async build() {
    const compiler = rspack({ context: fixture, mode: 'development', entry: './entry.js' });
    const fn = workerFunction(path.join(fixture, 'fail.cjs'), {});
    expect(() => compiler.hooks.make.tapPromise('unsupported', fn)).toThrow(/only supports/);
    compiler.hooks.normalModuleFactory.tap('validate', factory => {
      expect(() => factory.hooks.beforeResolve.tap('sync', fn)).toThrow(/only supports/);
      expect(() => factory.hooks.beforeResolve.tapAsync('async', fn)).toThrow(/only supports/);
      expect(() => factory.hooks.afterResolve.tapPromise('unsupported', fn)).toThrow(/only supports/);
      expect(() => factory.hooks.resolveForScheme.for('file').tapPromise('map', fn)).toThrow(/only supports/);
      factory.hooks.beforeResolve.intercept({ call() {} });
      expect(() => factory.hooks.beforeResolve.tapPromise('intercepted', fn)).toThrow(/interceptors/);
    });
    try { expect((await run(compiler)).hasErrors()).toBe(false); } finally { await close(compiler); }
  },
}, ...[
  ['throw', /worker function failure/],
  ['crash', /worker dropped/],
  ['invalid', /must return false or undefined/],
  ['conversion', /String|string/],
  ['export', /must export a function/],
  ['missing', /Cannot resolve workerFunction/],
].map(([kind, expected]) => ({
  description: `workerFunction ${kind} fails without hanging and permits subsequent builds`,
  async build() {
    const compiler = rspack({
      context: fixture, mode: 'development', entry: './entry.js',
      plugins: [{ apply(compiler) {
        compiler.hooks.normalModuleFactory.tap('fail', factory => {
          const target = kind === 'export' ? 'not-function.cjs' : kind === 'missing' ? 'missing.cjs' : 'fail.cjs';
          factory.hooks.beforeResolve.tapPromise('worker', workerFunction(path.join(fixture, target), { kind }));
        });
      } }],
    });
    try {
      let message;
      try { message = (await run(compiler)).toString({ all: false, errors: true }); }
      catch (error) { message = error.message; }
      expect(message).toMatch(expected);
    } finally { await close(compiler); }
    const next = rspack({ context: fixture, mode: 'development', entry: './entry.js', plugins: [{ apply(compiler) {
      compiler.hooks.normalModuleFactory.tap('recover', factory => factory.hooks.beforeResolve.tapPromise('worker', workerFunction(path.join(fixture, 'fail.cjs'), {})));
    } }] });
    try { expect((await run(next)).hasErrors()).toBe(false); } finally { await close(next); }
  },
}))];


module.exports.push({
  description: 'workerFunction isolates shared descriptors across compilers and updates dynamic registrations across rebuilds',
  async build() {
    const shared = workerFunction('rewrite', { to: './entry.js', other: './other.js' });
    const outputs = [];
    const compilers = ['rewrite.cjs', 'rewrite.mjs'].map((file, index) => {
      const { createFsFromVolume, Volume } = require('memfs');
      const compiler = rspack({
        context: fixture, mode: 'development', entry: './virtual',
        output: { path: '/out' },
        resolveLoader: { alias: { rewrite: path.join(fixture, file) } },
      });
      compiler.outputFileSystem = createFsFromVolume(new Volume());
      let runNumber = 0;
      compiler.hooks.normalModuleFactory.tap('register', factory => {
        runNumber++;
        factory.hooks.beforeResolve.tap('main', () => {
          compiler.hooks.shouldEmit.tap(`dynamic-${runNumber}`, () => { outputs.push(index); });
        });
        factory.hooks.beforeResolve.tapPromise({ name: 'worker', stage: 10 }, shared);
      });
      return compiler;
    });
    try {
      for (let build = 0; build < 2; build++) {
        const stats = await Promise.all(compilers.map(run));
        expect(stats.every(item => !item.hasErrors())).toBe(true);
        expect(compilers[0].outputFileSystem.readFileSync('/out/main.js', 'utf8')).toContain('module.exports = 42');
        expect(compilers[1].outputFileSystem.readFileSync('/out/main.js', 'utf8')).toContain("module.exports = 'other'");
      }
      expect(outputs).toContain(0);
      expect(outputs).toContain(1);
    } finally { await Promise.all(compilers.map(close)); }
  },
});
