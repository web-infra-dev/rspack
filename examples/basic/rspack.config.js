const path = require('path');
/**
 * @type {import('webpack').Configuration}
 */
module.exports = (env) => {
  console.log('env:',env);
  return {
    context: __dirname,
    builtins: {
      noEmitAssets:false,
      html: [{
        template: './index.html'
      }],
      treeShaking: true,
      sideEffects: true
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
    output: {
      path: path.resolve(__dirname, 'dist')
    }
  }
};
