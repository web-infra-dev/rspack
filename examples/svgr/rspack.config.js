/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
  entry: {
    main: './index.jsx'
  },
  module: {
    rules: [{
      test: /\.svg$/,
      use: [
        {
          loader: '@svgr/webpack',
        },
        {
          loader: 'url-loader'
        }
      ],
      type: 'js'
    }]
  },
  builtins: {
    html: [{
      template: './index.html'
    }]
  }
}