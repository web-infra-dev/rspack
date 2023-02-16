const CopyPlugin = require('copy-webpack-plugin');
/*
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
  entry: {
    main: './src/index.jsx'
  },
  builtins: {
    html: [{
      template: './index.html'
    }],
    react: {
      'runtime': 'automatic'
    }
  },
  plugins: [new CopyPlugin([{
    from: "public",
    to: "."
  }])]
}