module.exports = {
  mode : 'development',
  entry : {
    main : './src/index.js',
  },
  output : {
    // publicPath : 'http://localhost:3000',
    // filename: '[name].[contenthash:8][ext]',
  },
  devServer: {
    webSocketServer: true,
    hot: true,
  },
  module : {
    rules : [
      {
        test : '.less',
        type : 'css'
      }, 
      {
        // use entry or not
        test: "\.js$",
        uses: [
          {
            builtinLoader: "react-refresh-loader"
          }
        ]
      }
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
    }
  },
};
