import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';

const pluginName = 'EquivalentGlobRoot';

export default defineConfig({
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.contextModuleFactory.tap(pluginName, (factory) => {
          factory.hooks.afterResolve.tap(pluginName, (data) => {
            if (data !== false && data.request.includes('?equivalent-root')) {
              // Preserve the parent segment in the hook's resource.
              data.resource += `${path.sep}..${path.sep}outside`;
            }
          });
        });
      },
    }),
  ],
});
