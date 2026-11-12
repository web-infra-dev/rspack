import { defineConfig, definePlugin } from '@rspack/cli';

let step = 0;
let factorizeRequests: string[] = [];

export default defineConfig({
  plugins: [
    definePlugin((compiler) => {
      const PLUGIN_NAME = 'TEST_PLUGIN';
      const { EntryPlugin } = compiler.rspack;
      compiler.hooks.finishMake.tapPromise(PLUGIN_NAME, (compilation) => {
        return new Promise<void>((resolve, reject) => {
          const dependency = EntryPlugin.createDependency('./foo.js');
          compilation.addInclude(compiler.context, dependency, {}, (err) => {
            if (err) return reject(err);
            const module = compilation.moduleGraph.getModule(dependency);
            expect(module).toBeTruthy();
            return resolve();
          });
        });
      });

      compiler.hooks.compilation.tap(
        PLUGIN_NAME,
        (_compilation, { normalModuleFactory }) => {
          normalModuleFactory.hooks.factorize.tap(PLUGIN_NAME, (data) => {
            factorizeRequests.push(data.request);
          });
        },
      );
      compiler.hooks.done.tap(PLUGIN_NAME, () => {
        if (step === 0) {
          expect(factorizeRequests.length).toBe(2);
          expect(factorizeRequests.includes('./index.js')).toBe(true);
          expect(factorizeRequests.includes('./foo.js')).toBe(true);
        } else if (step === 1) {
          expect(factorizeRequests.length).toBe(1);
          expect(factorizeRequests.includes('./index.js')).toBe(true);
        } else {
          throw new Error('Unexpected step');
        }
        step += 1;
        factorizeRequests = [];
      });
    }),
  ],
});
