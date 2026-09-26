const { RuntimeModule } = require('@rspack/core');

const PLUGIN_NAME = 'OnChunksLoadedPriorityTestPlugin';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'web',
  output: {
    environment: {
      logicalAssignment: true,
    },
  },
  plugins: [
    (compiler) => {
      const { RuntimeGlobals } = compiler.rspack;

      class DeferredPriorityRuntimeModule extends RuntimeModule {
        constructor() {
          super('deferred priority probe', RuntimeModule.STAGE_TRIGGER);
        }

        generate() {
          const order = 'globalThis.__onChunksLoadedOrder';
          return [
            order + ' = [];',
            RuntimeGlobals.onChunksLoaded +
              '(0, ["__never_loaded__"], function() { ' +
              order +
              '.push("blocked"); }, 0);',
            RuntimeGlobals.onChunksLoaded +
              '(0, [' +
              JSON.stringify(this.chunk.id) +
              '], function() { ' +
              order +
              '.push("even"); }, 2);',
          ].join('\n');
        }
      }

      compiler.hooks.thisCompilation.tap(PLUGIN_NAME, (compilation) => {
        compilation.hooks.additionalTreeRuntimeRequirements.tap(
          PLUGIN_NAME,
          (chunk, set) => {
            set.add(RuntimeGlobals.onChunksLoaded);
            compilation.addRuntimeModule(
              chunk,
              new DeferredPriorityRuntimeModule(),
            );
          },
        );
      });
    },
  ],
};
