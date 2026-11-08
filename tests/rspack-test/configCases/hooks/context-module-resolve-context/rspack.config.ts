import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';
import path from 'node:path';

const pluginName = 'context-module-resolve-context';

class ContextModuleResolveContextPlugin {
  apply(compiler: Compiler) {
    compiler.hooks.contextModuleFactory.tap(
      pluginName,
      (contextModuleFactory) => {
        contextModuleFactory.hooks.beforeResolve.tap(
          pluginName,
          (resolveData) => {
            if (resolveData === false) return false;
            expect(resolveData.context).toBe(
              path.join(import.meta.dirname, 'src'),
            );
            expect(
              path.isAbsolute(resolveData.request.split(/[?#]/, 1)[0]),
            ).toBe(false);
            if (
              resolveData.request.includes('after-source') ||
              resolveData.request.includes('after-resource-override')
            ) {
              return;
            }
            if (resolveData.request.includes('request-override')) {
              resolveData.context = path.join(import.meta.dirname, 'fixtures');
              resolveData.request = '../src/request-override';
              resolveData.recursive = true;
              return;
            }
            if (resolveData.request.includes('../shared')) {
              resolveData.context = path.join(
                import.meta.dirname,
                'fixtures',
                'nested',
              );
              resolveData.recursive = false;
              return;
            }
            resolveData.context = path.join(import.meta.dirname, 'fixtures');
          },
        );
        contextModuleFactory.hooks.beforeResolve.tap(
          `${pluginName}-observer`,
          (resolveData) => {
            if (resolveData === false) return false;
            if (resolveData.request.includes('../src/request-override')) {
              expect(resolveData.context).toBe(
                path.join(import.meta.dirname, 'fixtures'),
              );
              expect(resolveData.recursive).toBe(true);
              return;
            }
            if (resolveData.request.includes('../shared')) {
              expect(resolveData.context).toBe(
                path.join(import.meta.dirname, 'fixtures', 'nested'),
              );
              expect(resolveData.recursive).toBe(false);
            }
          },
        );
        contextModuleFactory.hooks.afterResolve.tap(
          pluginName,
          (resolveData) => {
            if (resolveData === false) return false;
            if (resolveData.request.includes('after-resource-override')) {
              expect(resolveData.context).toBe(
                path.join(import.meta.dirname, 'src'),
              );
              resolveData.resource = path.join(
                import.meta.dirname,
                'fixtures',
                'after-resource',
              );
              return;
            }
            if (resolveData.request.includes('after-source')) {
              expect(resolveData.context).toBe(
                path.join(import.meta.dirname, 'src'),
              );
              resolveData.context = path.join(import.meta.dirname, 'fixtures');
              return;
            }
            if (resolveData.request.includes('../src/request-override')) {
              expect(resolveData.context).toBe(
                path.join(import.meta.dirname, 'fixtures'),
              );
              expect(resolveData.recursive).toBe(true);
              return;
            }
            if (resolveData.request.includes('../shared')) {
              expect(resolveData.context).toBe(
                path.join(import.meta.dirname, 'fixtures', 'nested'),
              );
              expect(resolveData.recursive).toBe(false);
              return;
            }
            expect(resolveData.context).toBe(
              path.join(import.meta.dirname, 'fixtures'),
            );
          },
        );
      },
    );
  }
}

export default defineConfig({
  context: import.meta.dirname,
  entry: './src/index.js',
  plugins: [new ContextModuleResolveContextPlugin()],
});
