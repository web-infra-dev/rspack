import fs from 'node:fs';
import path from 'node:path';
import readDir from './readdir.js';
import sources from 'webpack-sources';

const { RawSource } = sources;

/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    clean: true,
  },
  plugins: [
    (compiler) => {
      let once = true;
      compiler.hooks.thisCompilation.tap('Test', (compilation) => {
        compilation.hooks.processAssets.tap('Test', (assets) => {
          if (once) {
            const outputPath = compilation.getPath(compiler.outputPath, {});
            const customDir = path.join(
              outputPath,
              'this/dir/should/be/removed',
            );
            fs.mkdirSync(customDir, { recursive: true });
            fs.writeFileSync(path.join(customDir, 'file.ext'), '');
            once = false;
          }
          assets['this/dir/should/not/be/removed/file.ext'] = new RawSource('');
        });
      });
      compiler.hooks.afterEmit.tap('Test', (compilation) => {
        const outputPath = compilation.getPath(compiler.outputPath, {});
        expect(readDir(outputPath)).toMatchInlineSnapshot(`
			Object {
			  directories: Array [
			    this,
			    this/dir,
			    this/dir/should,
			    this/dir/should/not,
			    this/dir/should/not/be,
			    this/dir/should/not/be/removed,
			  ],
			  files: Array [
			    this/dir/should/not/be/removed/file.ext,
			    bundle0.js,
			  ],
			}
		`);
      });
    },
  ],
};
