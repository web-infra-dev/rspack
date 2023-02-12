const path = require('path');
module.exports = (env) => {
  console.log('env:',env);
  /**
 * @type {import('webpack').Configuration}
 */
  const config =  {
    mode: 'development',
    context: __dirname,
    builtins: {
      noEmitAssets:false,
      html: [{
        template: './index.html'
      }],
      treeShaking: true,
      minify: false
    },
    context: __dirname,
    entry: {
      main: {
        import: ["./index.js"],
      }
    },
    define: {
      "process.env.NODE_ENV": "'development'",
    },
    infrastructureLogging: {
      debug: true
    },
    optimization: {
      sideEffects: true
    },
    output: {
      path: path.resolve(__dirname, 'dist')
    },
  }
  return config
};
