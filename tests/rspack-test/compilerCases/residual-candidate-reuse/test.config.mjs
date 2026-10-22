import assert from 'node:assert/strict';
import path from 'node:path';
import { runCompiler } from '@rspack/test-tools/helper/lifecycle';

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description: 'keeps the remaining candidate after an overlapping extraction',
  options: () => ({
    mode: 'production',
    entry: './index.js',
    cache: false,
    incremental: false,
    devtool: false,
    optimization: {
      minimize: false,
      concatenateModules: false,
      splitChunks: {
        chunks: 'all',
        dedupDepth: 1,
        minSize: 50,
        minSizeReduction: 0,
        cacheGroups: { defaultVendors: false },
      },
    },
  }),
  async build(_context, compiler) {
    const stats = await runCompiler(compiler);
    assert.equal(stats.hasErrors(), false, stats.toString());
    const { compilation } = stats;
    const placements = { 'm1.js': [], 'm2.js': [], 'p.js': [] };
    for (const chunk of compilation.chunks) {
      const consumers = [...chunk.groupsIterable]
        .map((group) => group.name)
        .sort()
        .join('');
      for (const module of compilation.chunkGraph.getChunkModules(chunk)) {
        const name = path.basename(module.nameForCondition?.() || '');
        placements[name]?.push(consumers);
      }
    }
    for (const groups of Object.values(placements)) groups.sort();
    // CDE extracts m1 + p first. The original ABC candidate can then shrink
    // to AB and extract m1 + m2 together, without making C load that chunk.
    assert.deepEqual(placements, {
      'm1.js': ['AB', 'CDE'],
      'm2.js': ['AB', 'C'],
      'p.js': ['CDE'],
    });
  },
};
