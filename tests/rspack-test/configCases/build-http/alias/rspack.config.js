const os = require('node:os');
const path = require('node:path');

const tempDir = path.join(os.tmpdir(), 'rspack-build-http-alias');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  resolve: {
    alias: {
      'remote-runtime': 'https://test.rspack.rs/runtime.js',
    },
  },
  experiments: {
    buildHttp: {
      allowedUris: ['https://test.rspack.rs/'],
      cacheLocation: false,
      lockfileLocation: path.join(tempDir, `lock-${process.pid}.json`),
      httpClient: require('./custom-http-client'),
    },
  },
};
