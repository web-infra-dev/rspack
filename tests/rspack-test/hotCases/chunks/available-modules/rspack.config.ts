import assert from 'node:assert/strict';
import { defineConfig, definePlugin } from '@rspack/cli';
export default defineConfig({
  mode: 'development',
  cache: true,
  incremental: true,
  optimization: {
    splitChunks: false,
    concatenateModules: false,
    usedExports: false,
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        let build = 0;
        compiler.hooks.done.tap('AssertAvailableRebuild', (stats) => {
          const c = stats.compilation;
          const modules = c.namedChunkGroups
            .get('child')!
            .chunks.flatMap((chunk) => [
              ...c.chunkGraph.getChunkModulesIterable(chunk),
            ]);
          assert.equal(
            modules.filter((m) => /[\\/]m\.js$/.test(m.identifier())).length,
            build === 1 ? 1 : 0,
          );
          build++;
        });
      },
    }),
  ],
});
