import { defineConfig, definePlugin } from '@rspack/cli';
import path from 'node:path';
import readDir from './readdir.ts';
import { rspack } from '@rspack/core';

export default defineConfig((_, { testPath }) => {
  return {
    output: {
      clean: true,
    },
    plugins: [
      new rspack.DllPlugin({
        name: '[name]_dll',
        path: path.resolve(testPath, 'manifest.json'),
      }),
      definePlugin((compiler) => {
        compiler.hooks.afterEmit.tap('Test', (compilation) => {
          const outputPath = compilation.getPath(compiler.outputPath, {});
          expect(readDir(outputPath)).toMatchInlineSnapshot(`
						Object {
						  directories: Array [],
						  files: Array [
						    bundle0.js,
						    manifest.json,
						  ],
						}
					`);
        });
      }),
    ],
  };
});
