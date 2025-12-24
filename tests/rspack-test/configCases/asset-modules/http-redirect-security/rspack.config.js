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
