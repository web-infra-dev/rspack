/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
  {
    externals: {
      './tailwind_min_css.bundle0.js': 'commonjs ./tailwind_min_css.bundle0.js',
      './tailwind_min_css.bundle1.js': 'commonjs ./tailwind_min_css.bundle1.js',
      './use-style_js.bundle0.js': 'commonjs ./use-style_js.bundle0.js',
      './use-style_js.bundle1.js': 'commonjs ./use-style_js.bundle1.js',
    },
    target: 'web',
    mode: 'development',
    output: {
      uniqueName: 'my-app',
    },
    optimization: {
      chunkIds: 'named',
    },
    module: {
      rules: [
        {
          test: /\.css$/,
          type: 'css/auto',
        },
      ],
    },
  },
  {
    externals: {
      './tailwind_min_css.bundle0.js': 'commonjs ./tailwind_min_css.bundle0.js',
      './tailwind_min_css.bundle1.js': 'commonjs ./tailwind_min_css.bundle1.js',
      './use-style_js.bundle0.js': 'commonjs ./use-style_js.bundle0.js',
      './use-style_js.bundle1.js': 'commonjs ./use-style_js.bundle1.js',
    },
    target: 'web',
    mode: 'production',
    optimization: {
      chunkIds: 'named',
    },
    performance: false,
    module: {
      rules: [
        {
          test: /\.css$/,
          type: 'css/auto',
        },
      ],
    },
  },
];
