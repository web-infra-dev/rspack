const assert = require('node:assert/strict');
const { Worker, isMainThread } = require('node:worker_threads');
const rspack = require('@rspack/core');
const { createFsFromVolume, Volume } = require('memfs');
const { closeCompiler, createGCTracker, forceGC, runCompiler } = require('./helpers.cjs');

async function exercise() {
  for (const mode of ['normal', 'hook-error', 'runner-error', 'builtin-only', 'no-loaders']) {
    const tracker = createGCTracker();
    const loader = require.resolve('./lifecycle-loader.cjs');
    const use = mode === 'no-loaders' ? [] : mode === 'builtin-only'
      ? ['builtin:swc-loader'] : [loader, 'builtin:swc-loader', loader];
    let build = 0;
    let nativeCalls = 0;
    let native;
    let facade;
    const compiler = rspack({
      context: __dirname,
      mode: 'development',
      cache: false,
      experiments: { incremental: false },
      entry: () => `./entry.js?build=${build}`,
      output: { path: '/', filename: 'bundle.js' },
      module: { rules: [{ test: /entry\.js$/, use }] },
      plugins: [{
        apply(compiler) {
          compiler.hooks.beforeRun.tap('LoaderLifecycle', () => {
            build++;
            nativeCalls = 0;
            native = undefined;
            facade = undefined;
          });
          compiler.hooks.compilation.tap('LoaderLifecycle', compilation => {
            const hook = compiler.webpack.NormalModule.getCompilationHooks(compilation).loader;
            hook.tap({ name: 'LoaderLifecycle', stage: -10 }, context => {
              tracker.track(context, `${build}:facade`);
              facade = new WeakRef(context);
              context.fromHook = 'preserved';
              context.hookSelf = () => context;
              if (mode === 'hook-error') throw new Error('lifecycle hook error');
            });
            hook.tap({ name: 'LoaderLifecycle', stage: 10 }, context => {
              assert.equal(context, facade.deref());
            });
          });
          const plugin = compiler.__internal__builtinPlugins.find(p => p.name === 'JsLoaderRspackPlugin');
          const run = plugin.options;
          plugin.options = async context => {
            nativeCalls++;
            if (native) assert.equal(context, native.deref());
            else {
              native = new WeakRef(context);
              tracker.track(context, `${build}:native`);
            }
            // Both native and facade identities must survive GC between entries.
            await forceGC(2);
            assert(facade.deref());
            if (mode === 'runner-error') throw new Error('lifecycle runner error');
            return run(context);
          };
        },
      }],
    });
    compiler.outputFileSystem = createFsFromVolume(new Volume());
    try {
      for (let i = 0; i < 2; i++) {
        const stats = await runCompiler(compiler);
        assert.equal(stats.hasErrors(), mode.endsWith('error'));
        if (mode === 'normal') assert(nativeCalls > 1, `build ${build}: ${nativeCalls} native calls`);
        // Collect before another build or compiler.close: guard destruction
        // must actively release the final batch, including hook-only failures.
        await tracker.waitForCollection(`${build}:facade`);
        if (native) await tracker.waitForCollection(`${build}:native`);
      }
    } finally {
      await closeCompiler(compiler);
    }
  }
}

async function main() {
  if (!isMainThread) return exercise();
  // Independent N-API environments must never reuse each other's references.
  const worker = new Worker(__filename);
  await Promise.all([
    exercise(),
    new Promise((resolve, reject) => {
      worker.on('error', reject);
      worker.on('exit', code => code === 0 ? resolve() : reject(new Error(`worker exited ${code}`)));
    }),
  ]);
  // A worker's environment cleanup must not disable the surviving environment.
  await exercise();
}

main().catch(error => {
  console.error(error);
  process.exitCode = 1;
});
