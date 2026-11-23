import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';

// Copied unchanged from webpack's html/build-http-absolute-url case.
const fixture = path.join(import.meta.dirname, 'lock-files');
const expected = fs.readFileSync(path.join(fixture, 'lock.json'), 'utf8');
const url = Object.keys(JSON.parse(expected)).find((key) => key !== 'version')!;
const cache = path.join(fixture, 'test');
const content = fs.readFileSync(
  path.join(
    cache,
    'https_raw.githubusercontent.com',
    fs.readdirSync(path.join(cache, 'https_raw.githubusercontent.com'))[0],
  ),
);

export default [true, false].map((frozen) => {
  const lockfileLocation = frozen
    ? path.join(fixture, 'lock.json')
    : path.join(import.meta.dirname, 'written.lock');
  if (!frozen) fs.writeFileSync(lockfileLocation, '{"version":1}');
  let requests = 0;
  return defineConfig({
    name: frozen ? 'read-webpack-lockfile' : 'write-webpack-lockfile',
    mode: 'production',
    target: 'web',
    output: { publicPath: '' },
    module: { rules: [{ test: /\.svg$/, type: 'asset/resource' }] },
    experiments: {
      buildHttp: {
        allowedUris: ['https://raw.githubusercontent.com/'],
        lockfileLocation,
        cacheLocation: frozen ? cache : false,
        frozen,
        httpClient: async (request) => {
          assert.equal(
            frozen,
            false,
            'The webpack content cache must work offline',
          );
          assert.equal(request, url);
          requests++;
          return {
            status: 200,
            headers: {
              'content-type': 'image/svg+xml',
              'cache-control': 'max-age=3600',
              etag: '"transient-validator"',
            },
            body: content,
          };
        },
      },
    },
    plugins: [
      definePlugin((compiler) => {
        compiler.hooks.done.tap('CheckWebpackLockfile', (stats) => {
          assert.equal(stats.hasErrors(), false);
          assert.equal(fs.readFileSync(lockfileLocation, 'utf8'), expected);
          assert.equal(frozen ? requests === 0 : requests > 0, true);
        });
      }),
    ],
  });
});
