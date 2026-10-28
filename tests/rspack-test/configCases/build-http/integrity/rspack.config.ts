import { createHash } from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import { DefinePlugin, type HttpUriOptions } from '@rspack/core';
import cases from './cases.ts';

type HttpClient = NonNullable<HttpUriOptions['httpClient']>;

const url = 'http://integrity.example/module.js';
const source = (value: string) => `export default ${JSON.stringify(value)};\n`;
const integrity = (content: string) =>
  `sha512-${createHash('sha512').update(content).digest('base64')}`;

// These fixtures use the existing Rspack lockfile format and cache filenames.
const cachePath = (directory: string, resolved: string) =>
  path.join(
    directory,
    'http___integrity.example',
    `_${path.basename(new URL(resolved).pathname, '.js')}_${createHash('sha512').update(resolved).digest('hex').slice(0, 20)}.js`,
  );

export default cases.map((test) => {
  const directory = path.join(import.meta.dirname, 'fixtures', test.name);
  const lockfileLocation = path.join(directory, 'rspack.lock');
  const cacheLocation = path.join(directory, 'rspack.lock.data');
  const resolved = new URL(test.resolved || 'module.js', url).href;
  const lockedUrl = new URL(test.lockAt || 'module.js', url).href;
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
              [lockedUrl]:
                typeof test.locked === 'string' ? test.locked : entry,
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
  const requests = new Map<string, Set<string | undefined>>();
  const frozen = test.frozen ?? test.mode !== 'development';
  return defineConfig({
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
        httpClient: async (request, headers): ReturnType<HttpClient> => {
          const validators =
            requests.get(request) || new Set<string | undefined>();
          validators.add(headers['if-none-match']);
          requests.set(request, validators);
          const redirectIndex = redirects.indexOf(request);
          if (redirectIndex >= 0 && redirectIndex < redirects.length - 1) {
            return {
              status: 302,
              headers: {
                location: redirects[redirectIndex + 1],
                ...(test.redirectNoCache
                  ? { 'cache-control': 'no-cache' }
                  : {}),
              },
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
      definePlugin((compiler) => {
        compiler.hooks.done.tap('CheckHttpIntegrity', (stats) => {
          expect(stats.hasErrors(), test.name).toBe(Boolean(test.error));
          const afterLock = fs.existsSync(lockfileLocation)
            ? fs.readFileSync(lockfileLocation, 'utf8')
            : undefined;
          const afterCache = fs.existsSync(contentPath)
            ? fs.readFileSync(contentPath, 'utf8')
            : undefined;
          if (frozen || test.error) {
            expect(
              afterLock,
              `${test.name}: lockfile must remain unchanged`,
            ).toBe(beforeLock);
            expect(
              afterCache,
              `${test.name}: cache must remain unchanged`,
            ).toBe(beforeCache);
          } else {
            const updated = JSON.parse(afterLock!).entries[url];
            if (test.noCache) {
              expect(updated).toBe('no-cache');
            } else {
              expect(updated.integrity).toBe(
                integrity(source(test.expected || 'trusted')),
              );
              expect(updated.resolved).toBe(redirects.at(-1) || resolved);
              expect(
                fs.readFileSync(
                  cachePath(cacheLocation, updated.resolved),
                  'utf8',
                ),
              ).toBe(source(test.expected || 'trusted'));
            }
          }
          if (
            (beforeCache !== undefined && !test.upgrade && !test.lockAt) ||
            test.invalidLockfile
          ) {
            expect(
              requests.size,
              `${test.name}: unexpected network request`,
            ).toBe(0);
          } else {
            expect(
              requests.size,
              `${test.name}: expected a network request`,
            ).toBeGreaterThan(0);
          }
          if (test.requests) {
            expect(
              requests,
              `${test.name}: unexpected request URLs or validators`,
            ).toEqual(
              new Map(
                test.requests.map(([request, etag]) => [
                  new URL(request, url).href,
                  new Set([etag]),
                ]),
              ),
            );
          }
        });
      }),
    ],
  });
});
