module.exports = {
  mode : 'development',
  entry : {
    main : ['./src/index.js'],
  },
  output : {
    publicPath : '/',
    // filename: '[name].[contenthash:8][ext]',
  },
  devServer: {
    webSocketServer: true,
    hot: true,
  },
  module : {
    rules : [
      {
        test : {
          type: "regexp",
          matcher: '\\.less$'
        },
        type : 'css'
      }, 
    ],
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
    define : {
      'process.env.NODE_ENV' : "'development'"
    },
    progress: {},
    react: {
      development: true,
      refresh: true,
    }
  },
};
