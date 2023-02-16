const path = require('path');
module.exports = {
  context: __dirname,
  mode : 'development',
  entry : {
    main : ['./src/index.jsx'],
  },
  define : {
    'process.env.NODE_ENV' : '\'development\'',
  },
  builtins : {
    html : [{}],
  },
  module : {
    rules : [{test : /.less$/, use : [{loader : 'less-loader'}], type : 'css'}]
  },
  output: {
    path: path.resolve(__dirname,'dist')
  }
};
