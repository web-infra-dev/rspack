const assert = require('node:assert/strict');
const path = require('node:path');

const js = (name) => ({
  loader: path.resolve(__dirname, 'loader.js'),
  options: { name },
});
const builtin = { loader: 'builtin:swc-loader' };

module.exports = [
  ['js-only', [js('left'), js('right')]],
  ['js-builtin-js', [js('left'), builtin, js('right')]],
  ['builtin-only', [builtin]],
  ['no-loaders', []],
].map(([name, use]) => ({
  name,
  cache: false,
  module: { rules: [{ test: /value\.js$/, use }] },
  plugins: [
    {
      apply(compiler) {
        new compiler.webpack.HotModuleReplacementPlugin().apply(compiler);
        compiler.hooks.compilation.tap('LoaderHook', (compilation) => {
          const resource = path.resolve(__dirname, 'value.js');
          const dependency = path.resolve(__dirname, 'rspack.config.js');
          const events = (compiler.loaderHookEvents = []);
          const hook =
            compiler.webpack.NormalModule.getCompilationHooks(
              compilation,
            ).loader;
          for (const stage of [-10, 0, Infinity]) {
            hook.tap({ name: 'LoaderHook', stage }, (context, module) => {
              if (module.resource !== resource) return;
              events.push(stage);
              assert.equal(context.loaderIndex, 0);
              assert(
                context.loaders.every(
                  (loader) => !loader.pitchExecuted && !loader.normalExecuted,
                ),
              );
              // The native HMR tap runs at stage 0.
              if (stage === -10) {
                assert.equal(context.hot, false);
                context.hookLoaders = context.loaders;
                context.hookLoaderObjects = [...context.loaders];
                context.hookValue = 'from loader hook';
                context[Symbol.for('loader-hook-value')] = { value: 42 };
                Object.defineProperty(context, 'hookContext', {
                  get: () => context,
                });
                const addDependency = context.addDependency;
                context.addHookDependency = () => addDependency(dependency);
              } else {
                assert.equal(context.loaders, context.hookLoaders);
                context.loaders.forEach((loader, index) => {
                  assert.equal(loader, context.hookLoaderObjects[index]);
                });
                assert.equal(context.hookValue, 'from loader hook');
                assert.equal(context.hookContext, context);
                assert.equal(
                  context[Symbol.for('loader-hook-value')].value,
                  42,
                );
              }
              if (stage === Infinity) {
                assert.equal(context.hot, true);
                // JS writes must survive the native boundary too.
                context.hot = false;
                if (name === 'no-loaders') {
                  context.emitError(new Error('error from loader hook'));
                  context.emitWarning(new Error('warning from loader hook'));
                }
              }
              context.addDependency(dependency);
              context.cacheable(false);
            });
          }
          compilation.hooks.finishModules.tap('LoaderHook', (modules) => {
            assert.deepEqual(events, [
              -10,
              0,
              Infinity,
              ...(name.startsWith('js')
                ? ['pitch:left', 'pitch:right', 'normal:right', 'normal:left']
                : []),
            ]);
            const module = Array.from(modules).find(
              (module) => module.resource === resource,
            );
            assert(module.buildInfo.fileDependencies.has(dependency));
          });
          if (name === 'no-loaders') {
            compilation.hooks.processAssets.tap('LoaderHook', () => {
              assert.equal(compilation.errors.length, 1);
              assert.match(
                compilation.errors[0].message,
                /Module Error \(from \(not in loader scope\)\):.*error from loader hook/s,
              );
              assert.equal(compilation.warnings.length, 1);
              assert.match(
                compilation.warnings[0].message,
                /Module Warning \(from \(not in loader scope\)\):.*warning from loader hook/s,
              );
              compilation.errors = [];
              compilation.warnings = [];
            });
          }
        });
      },
    },
  ],
}));
