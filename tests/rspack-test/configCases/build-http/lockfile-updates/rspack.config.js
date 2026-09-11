const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const lockfileLocation = path.join(__dirname, 'rspack.lock');
module.exports = {
  target: 'web',
  experiments: {
    buildHttp: {
      allowedUris: ['http://updates.example/'],
      frozen: false,
      cacheLocation: false,
      lockfileLocation,
      httpClient: async (url) => ({
        status: 200,
        headers: { 'content-type': 'application/javascript' },
        body: Buffer.from(
          `export default ${Number(new URL(url).pathname.slice(1, -3))};`,
        ),
      }),
    },
  },
  plugins: [
    (compiler) => {
      compiler.hooks.done.tap('CheckHttpLockfileUpdates', () => {
        const lockfile = JSON.parse(fs.readFileSync(lockfileLocation, 'utf8'));
        assert.deepEqual(
          Object.keys(lockfile.entries).sort(),
          Array.from({ length: 6 }, (_, i) => `http://updates.example/${i}.js`),
        );
      });
    },
  ],
};
