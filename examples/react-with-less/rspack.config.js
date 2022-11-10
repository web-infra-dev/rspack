const lessLoader = require('@rspack/plugin-less').default;
module.exports = {
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
    rules : [{test : '.less$', uses : [{loader : lessLoader}], type : 'css'}]
  }
};
