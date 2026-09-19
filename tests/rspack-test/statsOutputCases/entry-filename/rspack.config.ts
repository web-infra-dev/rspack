import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: {
    a: './a.js',
    b: { import: './b.js', filename: 'c.js' },
  },
  profile: true,
  stats: {
    assets: true,
    reasons: true,
    chunks: true,
    chunkModules: true,
    dependentModules: true,
    chunkRelations: true,
    chunkOrigins: true,
    modules: false,
    publicPath: true,
  },
});
