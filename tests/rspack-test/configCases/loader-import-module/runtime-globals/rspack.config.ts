import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig(
  (['webpack', 'rspack'] as const).map((runtimeMode) => ({
    experiments: { runtimeMode },
    optimization: { concatenateModules: false },
    plugins: [
      definePlugin((compiler) => {
        const isRspack = compiler.options.experiments.runtimeMode === 'rspack';
        compiler.hooks.thisCompilation.tap('RuntimeScopes', (compilation) => {
          compilation.hooks.executeModule.tap('RuntimeScopes', (_, context) => {
            const requireFn =
              context[isRspack ? '__rspack_require' : '__webpack_require__'];
            const runtimeContext = context.__rspack_context;
            const scope = isRspack ? runtimeContext : requireFn;
            const otherScope = isRspack ? requireFn : runtimeContext;

            expect(scope).toHaveProperty('c');
            expect(scope).toHaveProperty('i');
            expect(otherScope).not.toHaveProperty('c');
            expect(otherScope).not.toHaveProperty('i');
            if (isRspack) {
              expect(runtimeContext).toHaveProperty('r', requireFn);
              expect(requireFn).not.toHaveProperty('r');
            }
          });
        });
      }),
    ],
  })),
);
