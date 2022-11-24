const lessLoader = require('@rspack/plugin-less').default;
const path = require('path');
module.exports = {
  context: __dirname,
  mode: 'development',
  entry: {
    main: ['./src/index.jsx'],
  },
  define: {
    'process.env.NODE_ENV': '\'development\'',
  },
  builtins: {
    html: [{}],
  },
  module: {
    rules: [{ test: /\.less$/, uses: [{ loader: lessLoader }], type: 'css' }]
  },
  resolve: {
    alias: {
      '~normalize.css': 'normalize.css'

    }
  },
  output: {
    path: path.resolve(__dirname, 'dist')
  }
};
