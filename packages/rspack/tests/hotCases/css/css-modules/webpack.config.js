module.exports = {
  entry: { 
    main: './index.js',
  },
  module: {
    rules: [
      {
        test: /\.module\.css$/,
        type: 'css/module'
      }
    ]
  }
}