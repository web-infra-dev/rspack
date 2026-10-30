import path from 'node:path';
export default ['builtin:less-loader', 'less-loader'].map((loader) => ({
  resolve: {
    alias: {
      './local': path.join(import.meta.dirname, 'aliased.less'),
      'alias-only': path.join(import.meta.dirname, 'extra.less'),
    },
  },
  module: {
    rules: [true, false, 'only'].map((webpackImporter) => ({
      test: /\.less$/,
      resourceQuery: new RegExp(`^\\?${webpackImporter}$`),
      type: 'asset/source',
      use: { loader, options: { webpackImporter } },
    })),
  },
}));
