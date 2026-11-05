import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import type { Configuration } from '@rspack/core';

const config = (concatenateModules: boolean): Configuration => ({
  mode: 'production',
  optimization: {
    sideEffects: true,
    providedExports: true,
    usedExports: true,
    concatenateModules,
    minimize: false,
  },
  plugins: [
    definePlugin((compiler) => {
      compiler.hooks.done.tap('Test', (stats) => {
        const resources = new Set<string>();
        for (const module of stats.compilation.modules) {
          const resource = (module as { resource?: string }).resource;
          if (resource) resources.add(resource);
          for (const inner of module.modules ?? []) {
            const innerResource = (inner as { resource?: string }).resource;
            if (innerResource) resources.add(innerResource);
          }
        }
        expect(
          resources.has(
            path.resolve(import.meta.dirname, 'lib/Button/Button.mjs'),
          ),
        ).toBe(true);
      });
    }),
  ],
});

export default defineConfig([config(false), config(true)]);
