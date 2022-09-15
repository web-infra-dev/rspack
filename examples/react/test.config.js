module.exports = {
  mode : 'development',
  entry : {
    main : './src/index.js',
  },
  output : {
    publicPath : 'http://localhost:3000',
    filename: '[name].[chunkhash:8][ext]',
  },
  define : {
    'process.env.NODE_ENV' : 'development',
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
    html : [{}],
  },
};
