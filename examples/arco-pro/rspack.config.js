const lessLoader = require('@rspack/plugin-less').default;
const postcssLoader = require('@rspack/plugin-postcss');
const path = require('path');
const { transform } = require('@svgr/core');
async function svgLoader(loaderContext){
  const svgCode = loaderContext.source.getCode();
  const filePath = loaderContext.resourcePath;
  const componentCode = await transform(svgCode, {}, {
    filePath,
    caller: {
      previousExport: null
    }
  })
  return {
    content: componentCode,
    meta: "",
    map: undefined
  }
}
/**
 * @type {import('webpack').Configuration}
 */
module.exports = {
  mode : 'development',
  context : __dirname,
  entry : {main : './src/index.tsx'},
  devServer: {
    port: 5555
  },
  devtool: 'source-map',
  builtins : {
    html : [{
           template : './index.html',
           publicPath: '/'
         }],
    define : {'process.env.NODE_ENV' : JSON.stringify('development')},
    react: {
      development: true,
      refresh: true,
    }
  },
  module : {
    rules :
    [
      {
        test : 'module.less$',
        uses :
             [
               {loader : lessLoader},
               {loader : postcssLoader, options : {modules : true}}
             ],
        type : 'css'
      },
      {test : '.less$', uses : [{loader : lessLoader}], type : 'css'},
      {test : '.svg$', uses : [{loader : svgLoader}], type : 'jsx'}
    ]
  },
  resolve : {alias : {'@' : path.resolve(__dirname, 'src')}},
  output: {
    publicPath: '/'
  },
  infrastructureLogging: {
    debug:false
  },
  
}