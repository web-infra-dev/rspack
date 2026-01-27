module.exports = {
  entry: {
    main: {
      import: "./index.js"
    },
    bundle: {
      import: './index.json'
    },
    cjs: {
      import: './cjs.js',
      filename: 'cjsBundle.mjs'
    }
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        type: 'javascript/auto'
      }
    ]
  },
  optimization: {
    splitChunks: {
      cacheGroups: {
        cjs: {
          test: /cjs\.js/,
          name: 'cjs'
        },
        json: {
          test: /index\.json/,
          name: 'json'
        }
      }
    }
  }
}