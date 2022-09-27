module.exports = {
  mode : 'development',
  entry : {
    main : './src/index.js',
  },
  output : {
    // publicPath : 'http://localhost:3000',
    // filename: '[name].[contenthash:8][ext]',
  },
  define : {
    'process.env.NODE_ENV' : 'development',
  },
  devServer: {
    webSocketServer: true
  },
  module : {
    rules : [{test : '.less',type : 'css',}],
    parser : {
      asset : {
        dataUrlCondition : {
          maxSize : 1,
        },
      },
    },
  },
  builtins : {
    html : [{
      template: './index.html'
    }],
  },
};
