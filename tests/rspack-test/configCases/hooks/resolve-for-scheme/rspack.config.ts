import { defineConfig, definePlugin } from '@rspack/cli';
import path from 'node:path';

const schemePath = path.join(import.meta.dirname, 'scheme.js');
const schemeQuery = '?match\u200b#literal';

export default defineConfig({
  module: {
    rules: [
      {
        test: (resourcePath) => resourcePath === schemePath,
        resourceQuery: schemeQuery,
        resourceFragment: '#match',
        type: 'asset/source',
      },
    ],
  },
  plugins: [
    definePlugin(function testPlugin(compiler) {
      compiler.hooks.compilation.tap(
        'testPlugin',
        (compilation, { normalModuleFactory }) => {
          normalModuleFactory.hooks.resolveForScheme
            .for('data')
            .tap('testPlugin', (resourceData) => {
              expect(resourceData.resource).toBe(
                'data:text/javascript;charset=utf-8,export const value = 42;',
              );
              expect(resourceData.path).toBe(
                'data:text/javascript;charset=utf-8,export const value = 42;',
              );
            });
          normalModuleFactory.hooks.resolveForScheme
            .for('probe')
            .tap('testPlugin', (resourceData) => {
              resourceData.resource = 'probe:display?raw#label';
              resourceData.path = schemePath;
              resourceData.query = schemeQuery;
              resourceData.fragment = '#match';
              return true;
            });
          compilation.hooks.finishModules.tap('testPlugin', (modules) => {
            const resolved = [...modules].find(
              (module) =>
                module instanceof compiler.webpack.NormalModule &&
                module.resource === 'probe:display?raw#label',
            );
            expect(resolved).toBeDefined();
          });
        },
      );
    }),
  ],
});
