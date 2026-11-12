import fs from 'node:fs';
import path from 'node:path';
import { rspack } from '@rspack/core';

export default [false, true].map((newCache) => {
  let generation = 0;
  return {
    name: newCache ? 'new' : 'legacy',
    description: 'emits the current DevTool comments and map on a cache hit',
    options(context) {
      return {
        context: context.getSource(),
        entry: './a.js',
        mode: 'development',
        devtool: false,
        cache: { type: 'memory' },
        experiments: { newCache },
        output: {
          path: context.getDist(newCache ? 'new' : 'legacy'),
          filename: '[name].[contenthash:8].js',
        },
        plugins: [
          new rspack.SourceMapDevToolPlugin({
            filename: '[file].map',
            append: () => `\n//# sourceMappingURL=[url]?build=${generation}`,
            moduleFilenameTemplate: ({ resourcePath }) =>
              `build-${generation}/${resourcePath}`,
          }),
        ],
      };
    },
    compiler(_context, compiler) {
      compiler.outputFileSystem = fs;
    },
    async build(_context, compiler) {
      for (generation = 0; generation < 2; generation++) {
        const stats = await new Promise((resolve, reject) =>
          compiler.run((error, stats) => (error ? reject(error) : resolve(stats))),
        );
        expect(stats.hasErrors()).toBe(false);
        const read = (file) =>
          fs.readFileSync(path.join(compiler.outputPath, file), 'utf8');
        const [js] = stats.compilation.entrypoints.get('main').getFiles();
        expect(read(js)).toContain(
          `sourceMappingURL=${js}.map?build=${generation}`,
        );
        expect(JSON.parse(read(`${js}.map`)).sources).toContain(
          `build-${generation}/a.js`,
        );
      }
    },
  };
});
