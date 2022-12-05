module.exports = {
  context: __dirname,
  entry: {
    main: './index.js'
  },
  devServer: {
    proxy: [
      {
        context: ['/api', '/auth'],
        target: 'http://localhost:3000'
      }
    ]
  }
}