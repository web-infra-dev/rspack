const path = require('node:path');
const fs = require('node:fs');
const loaderOptions = { cacheable: false, fail: false, builds: 0 };
let reused = 0;
let failHook = false;
let failValidHook = false;

/** @type {import('@rspack/test-tools').THookCaseConfig} */
module.exports = {
  description: 'rebuilds non-cacheable and failed modules before allowing a memory hit',
  snapshotFileFilter: () => false,
  options(context) {
    loaderOptions.dependency = path.join(context.getDist(), 'build-dependency.txt');
    return {
      context: __dirname,
      entry: './index.js',
      cache: { type: 'memory' },
      incremental: false,
      experiments: { newCache: { module: true } },
      module: { rules: [{ test: /index\.js$/, loader: './loader.js', options: loaderOptions }] },
      plugins: [compiler => {
        compiler.hooks.compilation.tap('CacheRecovery', compilation => {
          compilation.hooks.buildModule.tap('CacheRecovery', () => {
            if (failHook) {
              failHook = false;
              throw new Error('expected hook failure');
            }
          });
          compilation.hooks.stillValidModule.tap('CacheRecovery', () => {
            if (failValidHook) {
              failValidHook = false;
              throw new Error('expected cache-hit hook failure');
            }
            reused++;
          });
        });
      }],
    };
  },
  async compiler(context, compiler) {
    fs.mkdirSync(path.dirname(loaderOptions.dependency), { recursive: true });
    const setDependency = (content, age) => {
      fs.writeFileSync(loaderOptions.dependency, content);
      const time = new Date(Date.now() - age);
      fs.utimesSync(loaderOptions.dependency, time, time);
      compiler.inputFileSystem.purge();
    };
    setDependency('initial', 20000);
    const run = () => new Promise((resolve, reject) => compiler.run((error, stats) => {
      if (error) reject(error);
      else resolve(stats);
    }));
    expect((await run()).toJson({ all: false, errors: true }).errors).toEqual([]);
    expect(loaderOptions.builds).toBe(1);
    failHook = true;
    await expect(run()).rejects.toThrow('expected hook failure');
    expect(loaderOptions.builds).toBe(1);
    loaderOptions.cacheable = true;
    loaderOptions.fail = true;
    expect((await run()).hasErrors()).toBe(true);
    expect(loaderOptions.builds).toBe(2);
    loaderOptions.fail = false;
    expect((await run()).toJson({ all: false, errors: true }).errors).toEqual([]);
    expect(loaderOptions.builds).toBe(3);
    expect((await run()).toJson({ all: false, errors: true }).errors).toEqual([]);
    expect(loaderOptions.builds).toBe(3);
    expect(reused).toBe(1);
    setDependency('changed', 10000);
    expect((await run()).toJson({ all: false, errors: true }).errors).toEqual([]);
    expect(loaderOptions.builds).toBe(4);
    expect((await run()).toJson({ all: false, errors: true }).errors).toEqual([]);
    expect(loaderOptions.builds).toBe(4);
    expect(reused).toBe(2);
    failValidHook = true;
    await expect(run()).rejects.toThrow('expected cache-hit hook failure');
    expect(loaderOptions.builds).toBe(4);
    expect((await run()).toJson({ all: false, errors: true }).errors).toEqual([]);
    expect(loaderOptions.builds).toBe(5);
    expect((await run()).toJson({ all: false, errors: true }).errors).toEqual([]);
    expect(loaderOptions.builds).toBe(5);
    expect(reused).toBe(3);
  },
};
