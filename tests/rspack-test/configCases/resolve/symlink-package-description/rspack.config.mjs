import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const root = fs.realpathSync(
  fs.mkdtempSync(path.join(os.tmpdir(), 'rspack-resolver-symlink-')),
);
const issuer = path.join(root, 'issuer');
const target = path.join(root, 'target');
const modules = path.join(issuer, 'node_modules');
const linked = path.join(modules, 'linked');
const realFile = path.join(target, 'src/value.js');
const packageJson = path.join(issuer, 'package.json');

fs.mkdirSync(modules, { recursive: true });
fs.mkdirSync(path.dirname(realFile), { recursive: true });
fs.writeFileSync(packageJson, JSON.stringify({ name: 'issuer' }));
fs.writeFileSync(
  path.join(target, 'package.json'),
  JSON.stringify({ name: 'target' }),
);
fs.writeFileSync(realFile, 'module.exports = 42;');
fs.symlinkSync(path.dirname(realFile), linked, 'junction');

export default {
  resolve: { modules: [modules, 'node_modules'] },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.beforeRun.tapPromise(
          'SymlinkPackageDescription',
          async () => {
            for (const symlinks of [true, false]) {
              const resolver = compiler.resolverFactory.get('normal', {
                symlinks,
              });
              // Verify dependency tracking on both cold and cached resolutions.
              for (let attempt = 0; attempt < 2; attempt++) {
                const fileDependencies = new Set();
                const { resource, data } = await new Promise(
                  (resolve, reject) => {
                    resolver.resolve(
                      {},
                      issuer,
                      'linked/value.js',
                      { fileDependencies },
                      (error, resource, data) => {
                        if (error) reject(error);
                        else resolve({ resource, data });
                      },
                    );
                  },
                );
                assert.equal(
                  resource,
                  symlinks ? realFile : path.join(linked, 'value.js'),
                );
                // Like enhanced-resolve, retain the description found through
                // the link, even when the real target has a different ancestor.
                assert.equal(data.descriptionFileData.name, 'issuer');
                assert(fileDependencies.has(packageJson));
                assert(fileDependencies.has(resource));
              }
            }
          },
        );
        compiler.hooks.shutdown.tap('SymlinkPackageDescription', () => {
          fs.rmSync(root, { recursive: true, force: true });
        });
      },
    },
  ],
};
