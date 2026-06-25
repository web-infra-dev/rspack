/** @type {import("@rspack/core").Configuration} */
module.exports = [
  {
    externals: {
      fs: 'node-commonjs fs',
      path: 'node-commonjs path',
      './pseudo-export_style_module_css.bundle0.js':
        'commonjs ./pseudo-export_style_module_css.bundle0.js',
      './pseudo-export_style_module_css_exportsOnly.bundle0.js':
        'commonjs ./pseudo-export_style_module_css_exportsOnly.bundle0.js',
    },
    target: 'web',
    mode: 'development',
    module: {
      generator: {
        css: {
          exportsOnly: true,
        },
        'css/module': {
          exportsOnly: false,
        },
      },
      rules: [
        {
          resourceQuery: /\?module/,
          type: 'css/module',
        },
        // {
        // 	resourceQuery: /\?exportsOnly/,
        // 	generator: {
        // 		exportsOnly: true
        // 	},
        // 	type: "css/global"
        // },
        {
          test: /\.css$/,
          type: 'css/auto',
        },
      ],
    },

    node: {
      __dirname: false,
    },
  },
];
