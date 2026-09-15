const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');

const tempDir = path.join(os.tmpdir(), 'rspack-build-http-node-target');

fs.mkdirSync(tempDir, { recursive: true });

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  target: 'node',
  experiments: {
    buildHttp: {
      allowedUris: ['https://test.rspack.rs/'],
      cacheLocation: false,
      lockfileLocation: path.join(tempDir, `lock-${process.pid}.json`),
      httpClient: require('./custom-http-client'),
    },
  },
};
