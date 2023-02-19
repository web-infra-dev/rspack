/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
  target: 'node',
  mode: 'development',
  entry: {
    main: './src/index.js'
  },
  builtins: {
    minify:false
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        use: [{loader: 'babel-loader', options: {
          presets: [
            ['@babel/preset-env', { targets: 'defaults'}]
          ]
        }}]
      },
      {
        test: /\.less$/,
        use: [{loader: 'less-loader'}],
        type: 'css'
      },
      {
        test: /\.scss$/,
        use: [{loader:'sass-loader'}],
        type: 'css'
      },
      {
        test: /\.yaml$/,
        use: [{loader:'yaml-loader'}]
      },
      {
        test: /\.styl$/,
        use: [{loader:'stylus-loader'}],
        type: 'css'
      },
      {
        test: /\.mdx?$/,
        use: [
          {
            loader: '@mdx-js/loader',
            options: {}
          }
        ]
      }
    ]
  }
}