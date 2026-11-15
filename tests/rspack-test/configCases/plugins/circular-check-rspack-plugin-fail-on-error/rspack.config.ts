import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import { CircularCheckRspackPlugin } from '@rspack/core';

// Keep the expected error separate so the other circular-check configurations
// still receive the cache harness's error-free build reuse checks.
export default defineConfig({
  name: 'fail-on-error',
  mode: 'development',
  context: path.resolve(import.meta.dirname, '../circular-check-rspack-plugin'),
  entry: './deps/d.js',
  plugins: [
    new CircularCheckRspackPlugin({ failOnError: true }),
    (compiler) => {
      compiler.hooks.done.tap(
        'CircularCheckRspackPluginConfigCase',
        (stats) => {
          const { errors, warnings } = stats.toJson({
            all: false,
            errors: true,
            warnings: true,
          });
          expect(warnings).toHaveLength(0);
          expect(errors).toHaveLength(1);
          const message = errors![0].message.replaceAll('\\', '/');
          expect(message).toMatch(/Circular dependency detected/);
          expect(message).toContain(
            './deps/e.js -> ./deps/f.js -> ./deps/g.js -> ./deps/e.js',
          );
        },
      );
    },
  ],
});
