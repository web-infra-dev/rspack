import type { Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

class AssertEncodedMFRuntimeDataUriPlugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap(
      'AssertEncodedMFRuntimeDataUriPlugin',
      (_, { normalModuleFactory }) => {
        normalModuleFactory.hooks.beforeResolve.tap(
          'AssertEncodedMFRuntimeDataUriPlugin',
          (resolveData) => {
            const request = resolveData?.request;
            const prefix =
              '@module-federation/runtime/rspack.js!=!data:text/javascript,';

            if (!request || !request.startsWith(prefix)) {
              return;
            }

            const dataUriContent = request.slice(prefix.length);

            expect(dataUriContent).toMatch(/%[0-9A-Fa-f]{2}/);
            expect(dataUriContent).not.toContain(
              'import __module_federation_bundler_runtime__ from',
            );
          },
        );
      },
    );
  }
}

export default defineConfig({
  externals: {
    './container.js': 'commonjs ./container.js',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'container',
      filename: 'container.js',
      library: { type: 'commonjs-module' },
      exposes: ['./module'],
    }),
    new AssertEncodedMFRuntimeDataUriPlugin(),
  ],
});
