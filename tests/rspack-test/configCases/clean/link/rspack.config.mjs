import fs from 'node:fs';
import path from 'node:path';
import readDir from '../enabled/readdir.js';

/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    clean: true,
  },
  plugins: [
    (compiler) => {
      let once = true;
      compiler.hooks.environment.tap('Test', () => {
        if (once) {
          const outputPath = compiler.options.output.path;
          fs.mkdirSync(outputPath, { recursive: true });
          const originalPath = path.join(outputPath, 'file.ext');
          fs.writeFileSync(originalPath, '');
          const customDir = path.join(outputPath, 'this/dir/should/be/removed');
          fs.mkdirSync(customDir, { recursive: true });
          fs.symlinkSync(
            originalPath,
            path.join(customDir, 'file-link.ext'),
            'file',
          );
          once = false;
        }
      });
      compiler.hooks.afterEmit.tap('Test', (compilation) => {
        const outputPath = compilation.getPath(compiler.outputPath, {});
        expect(readDir(outputPath)).toMatchInlineSnapshot(`
					Object {
					  directories: Array [],
					  files: Array [
					    bundle0.js,
					  ],
					}
				`);
      });
    },
  ],
};
