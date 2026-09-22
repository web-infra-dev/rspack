import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  plugins: [
    definePlugin(function testPlugin(compiler) {
      compiler.hooks.compilation.tap(
        'testPlugin',
        (_compilation, { normalModuleFactory }) => {
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
        },
      );
    }),
  ],
});
