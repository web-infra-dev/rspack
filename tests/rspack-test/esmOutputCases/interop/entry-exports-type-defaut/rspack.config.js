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
        json: {
          test: /index\.json/,
          name: 'json'
        }
      }
    }
  }
}