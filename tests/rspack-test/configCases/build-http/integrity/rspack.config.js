const assert = require('node:assert/strict');
const { createHash } = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');
const { DefinePlugin } = require('@rspack/core');
const cases = require('./cases');

const url = 'http://integrity.example/module.js';
const source = (value) => `export default ${JSON.stringify(value)};\n`;
const integrity = (content) =>
  `sha512-${createHash('sha512').update(content).digest('base64')}`;

// These fixtures use the existing Rspack lockfile format and cache filenames.
const cachePath = (directory, resolved) =>
  path.join(
    directory,
    'http___integrity.example',
    `_${path.basename(new URL(resolved).pathname, '.js')}_${createHash('sha512').update(resolved).digest('hex').slice(0, 20)}.js`,
  );

module.exports = cases.map((test) => {
  const directory = path.join(__dirname, 'fixtures', test.name);
  const lockfileLocation = path.join(directory, 'rspack.lock');
  const cacheLocation = path.join(directory, 'rspack.lock.data');
  const resolved = new URL(test.resolved || 'module.js', url).href;
  const redirects = test.redirect
    ? [
        url,
        ...[test.redirect].flat().map((target) => new URL(target, url).href),
      ]
    : [];
  const contentPath = cachePath(cacheLocation, resolved);
  const entry = {
    resolved,
    integrity: integrity(source('trusted')),
    content_type: 'application/javascript',
    valid_until: test.fresh ? 8_000_000_000_000 : 0,
    etag: '"original"',
  };
  fs.mkdirSync(directory, { recursive: true });
  const beforeLock =
    test.locked === false
      ? undefined
      : test.invalidLockfile
        ? '{'
        : JSON.stringify({
            version: 1,
            entries: {
              [url]: typeof test.locked === 'string' ? test.locked : entry,
            },
          });
  if (beforeLock !== undefined) fs.writeFileSync(lockfileLocation, beforeLock);
  const beforeCache =
    test.cache === false || test.cached === false
      ? undefined
      : test.crlf
        ? source('trusted').replace(/\n/g, '\r\n')
        : source(test.cached || 'trusted');
  if (beforeCache !== undefined) {
    fs.mkdirSync(path.dirname(contentPath), { recursive: true });
    fs.writeFileSync(contentPath, beforeCache);
  }
  // Resolution and resource reading may request the same URL more than once.
  const requests = new Map();
  const frozen = test.frozen ?? test.mode !== 'development';
  return {
    name: test.name,
    mode: test.mode || 'production',
    target: 'web',
    experiments: {
      buildHttp: {
        allowedUris: ['http://integrity.example/'],
        lockfileLocation,
        cacheLocation: test.cache === false ? false : cacheLocation,
        ...(test.frozen === undefined ? {} : { frozen: test.frozen }),
        upgrade: test.upgrade || false,
        httpClient: async (request, headers) => {
          const validators = requests.get(request) || new Set();
          validators.add(headers['if-none-match']);
          requests.set(request, validators);
          const redirectIndex = redirects.indexOf(request);
          if (redirectIndex >= 0 && redirectIndex < redirects.length - 1) {
            return {
              status: 302,
              headers: { location: redirects[redirectIndex + 1] },
              body: Buffer.from(''),
            };
          }
          return {
            status: test.status || 200,
            headers: {
              'content-type': test.contentType || 'application/javascript',
              'cache-control': test.noCache ? 'no-cache' : 'max-age=3600',
              etag: '"refreshed"',
            },
            body: Buffer.from(
              test.status === 304 ? '' : source(test.remote || 'trusted'),
            ),
          };
        },
      },
    },
    plugins: [
      new DefinePlugin({
        EXPECTED_VALUE: JSON.stringify(test.expected || 'trusted'),
      }),
      (compiler) => {
        compiler.hooks.done.tap('CheckHttpIntegrity', (stats) => {
          assert.equal(stats.hasErrors(), Boolean(test.error), test.name);
          const afterLock = fs.existsSync(lockfileLocation)
            ? fs.readFileSync(lockfileLocation, 'utf8')
            : undefined;
          const afterCache = fs.existsSync(contentPath)
            ? fs.readFileSync(contentPath, 'utf8')
            : undefined;
          if (frozen || test.error) {
            assert.equal(
              afterLock,
              beforeLock,
              `${test.name}: lockfile must remain unchanged`,
            );
            assert.equal(
              afterCache,
              beforeCache,
              `${test.name}: cache must remain unchanged`,
            );
          } else {
            const updated = JSON.parse(afterLock).entries[url];
            if (test.noCache) {
              assert.equal(updated, 'no-cache');
            } else {
              assert.equal(
                updated.integrity,
                integrity(source(test.expected || 'trusted')),
              );
              assert.equal(updated.resolved, redirects.at(-1) || resolved);
              assert.equal(
                fs.readFileSync(
                  cachePath(cacheLocation, updated.resolved),
                  'utf8',
                ),
                source(test.expected || 'trusted'),
              );
            }
          }
          if (
            (beforeCache !== undefined && !test.upgrade) ||
            test.invalidLockfile ||
            (test.locked === false && frozen)
          ) {
            assert.equal(
              requests.size,
              0,
              `${test.name}: unexpected network request`,
            );
          } else {
            assert.ok(
              requests.size > 0,
              `${test.name}: expected a network request`,
            );
          }
          if (test.requests) {
            assert.deepEqual(
              requests,
              new Map(
                test.requests.map(([request, etag]) => [
                  new URL(request, url).href,
                  new Set([etag]),
                ]),
              ),
              `${test.name}: unexpected request URLs or validators`,
            );
          }
        });
      },
    ],
  };
});
