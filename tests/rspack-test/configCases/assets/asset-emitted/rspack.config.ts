import { defineConfig, definePlugin } from '@rspack/cli';
import { Compilation, sources } from '@rspack/core';

const Source = sources.Source;

export default defineConfig({
  plugins: [
    definePlugin((compiler) => {
      const files: Record<string, boolean> = {};
      compiler.hooks.assetEmitted.tap(
        'Test',
        (file, { content, source, outputPath, compilation, targetPath }) => {
          expect(Buffer.isBuffer(content)).toBe(true);
          expect(source).toBeInstanceOf(Source);
          expect(typeof outputPath).toBe('string');
          expect(typeof targetPath).toBe('string');
          expect(compilation).toBeInstanceOf(Compilation);
          files[file] = true;
        },
      );
      compiler.hooks.afterEmit.tap('Test', () => {
        expect(files).toMatchInlineSnapshot(`
			Object {
			  694.bundle0.js: true,
			  bundle0.js: true,
			}
		`);
      });
    }),
  ],
});
