module.exports = {
  entry: { 
    a: './a/index.js',
    b: './b/index.js',
    main: './main/index.js'
  },
  optimization: {
    runtimeChunk: 'single'
  }
}