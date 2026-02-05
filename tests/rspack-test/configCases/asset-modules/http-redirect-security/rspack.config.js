const ServerPlugin = require("./server");

const serverPlugin = new ServerPlugin(9991);

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'web',
  entry: './index.js',
  optimization: {
    moduleIds: 'named'
  },
  plugins: [
    serverPlugin,
    {
      apply(compiler){
        compiler.hooks.done.tap('test', (s)=>{
          console.log(s.toJson())

        })
      }
    }
  ],
  experiments: {
    buildHttp: {
      frozen: false,
      allowedUris: [
        "http://localhost:9991/"
      ],
    }
  }
};
