import fs from 'node:fs';
import path from 'node:path';
import rspack from '@rspack/core';

function run(compiler) {
  return new Promise((resolve, reject) => {
    compiler.run((error, stats) => {
      if (error) return reject(error);
      if (stats.hasErrors()) return reject(new Error(stats.toString()));
      resolve({
        hash: stats.hash,
        chunks: [...stats.compilation.chunks]
          .map(chunk => [chunk.id, chunk.hash, chunk.contentHash])
          .sort(([a], [b]) => String(a).localeCompare(String(b))),
      });
    });
  });
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description:
    'should give fresh compilers the same full hash after maxSize splitting',
  options(context) {
    const root = context.getDist();
    fs.mkdirSync(root, { recursive: true });
    fs.writeFileSync(
      path.join(root, 'index.js'),
      Array.from({ length: 3 }, (_, i) =>
        `import(/* webpackChunkName: "leaf${i}" */ './leaf${i}');`,
      ).join('\n'),
    );
    for (let i = 0; i < 3; i++) {
      fs.writeFileSync(
        path.join(root, `leaf${i}.js`),
        `require('./module${i}_0'); require('./module${i}_1');`,
      );
      for (let j = 0; j < 2; j++) {
        fs.writeFileSync(
          path.join(root, `module${i}_${j}.js`),
          `module.exports = "${i}_${j}_${'x'.repeat(90)}";`,
        );
      }
    }
    return {
      context: root,
      mode: 'development',
      devtool: false,
      entry: './index.js',
      output: { path: path.join(root, 'dist') },
      optimization: {
        moduleIds: 'named',
        chunkIds: 'named',
        splitChunks: {
          chunks: 'all',
          minSize: 0,
          maxSize: 180,
          cacheGroups: { default: false, defaultVendors: false },
        },
      },
    };
  },
  async build(_context, compiler) {
    const first = await run(compiler);
    const fresh = rspack(compiler.options);
    fresh.outputFileSystem = compiler.outputFileSystem;
    try {
      const second = await run(fresh);
      expect(second.chunks).toEqual(first.chunks);
      expect(second.hash).toBe(first.hash);
    } finally {
      await new Promise((resolve, reject) => {
        fresh.close(error => (error ? reject(error) : resolve()));
      });
    }
  },
};
