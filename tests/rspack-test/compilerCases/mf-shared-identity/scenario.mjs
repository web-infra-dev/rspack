import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { createRequire } from 'node:module';
import { rspack } from '@rspack/core';

const require = createRequire(import.meta.url);
export default async function run(scenario) {
  const paired = scenario.startsWith('paired-');
  const implementation = require.resolve(paired
    ? '@module-federation/runtime-tools-layers'
    : '@module-federation/runtime-tools');
  const bundlerRuntime = require.resolve('@module-federation/webpack-bundler-runtime', {
    paths: [implementation],
  });
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'rspack-shared-identity-'));
  const put = (file, source) => {
    const target = path.join(root, file);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, source);
  };
  for (const name of ['one', 'two', 'dependency']) {
    put('node_modules/' + name + '/package.json', JSON.stringify({ name, version: '1.0.0' }));
    put('node_modules/' + name + '/index.js', 'module.exports = ' + JSON.stringify(name));
  }
  const shared = {};
  const treeShaking = { mode: 'runtime-infer' };
  const config = {
    context: root,
    mode: 'development',
    target: 'async-node',
    entry: './index.js',
    devtool: false,
    cache: false,
    experiments: { layers: true },
    optimization: { minimize: false, concatenateModules: false },
    output: { path: path.join(root, 'dist'), filename: 'main.cjs', publicPath: root + '/dist/', library: { type: 'commonjs-module' } },
  };
  if (paired) {
    assert.equal(require(bundlerRuntime).bundlerRuntime.supportsSharedIdentity, true,
      'Value tests require the compatible Module Federation runtime, not a marker patched onto a legacy runtime');
  }
  let expectedError;
  if (scenario === 'unused-issuer-layer' || scenario === 'reachable-child-shared') {
    shared.one = { requiredVersion: false, treeShaking };
    shared['remote-only'] = { import: false, issuerLayer: 'server', requiredVersion: false };
    if (scenario === 'reachable-child-shared') {
      put('node_modules/one/index.js', "module.exports = require('dependency');");
      shared.dependency = { requiredVersion: false };
    }
    put('index.js', "module.exports = async () => (await import('one')).default;");
  } else if (scenario === 'consumer-layer') {
    shared['remote-only'] = { import: false, layer: 'server', requiredVersion: false };
    put('index.js', "module.exports = () => import('remote-only');");
    expectedError = /consumer layer "server"/;
  } else if (scenario === 'provider-layer' || scenario === 'paired-isolated-provider') {
    shared.one = { layer: 'server', requiredVersion: false };
    put('index.js', "const load = () => import('one'); module.exports = __webpack_require__.federation;");
    expectedError = paired ? undefined : /provider layer "server"/;
  } else {
    const layers = scenario.endsWith('layers');
    const differentVersions = scenario === 'safe-scopes';
    for (const [index, name] of ['one', 'two'].entries()) {
      shared[name] = {
        shareKey: 'key', requiredVersion: false, treeShaking,
        version: differentVersions && index ? '2.0.0' : '1.0.0',
        shareScope: layers ? 'default' : name,
        layer: layers ? name : undefined,
      };
    }
    put('index.js', "const imports = [() => import('one'), () => import('two')]; module.exports = __webpack_require__.federation;");
    if (!paired && !differentVersions) {
      expectedError = layers ? /provider layer/ : /distinct full fallbacks for version "1.0.0"/;
    }
  }
  config.plugins = [new rspack.container.ModuleFederationPlugin({
    name: 'identity_' + scenario.replaceAll('-', '_'), implementation, shared,
    library: { type: 'commonjs-module' },
    runtimePlugins: [path.join(import.meta.dirname, 'load-entry.cjs')],
  })];
  const compiler = rspack(config);
  try {
    const stats = await new Promise((resolve, reject) => compiler.run((error, stats) => error ? reject(error) : resolve(stats)));
    assert.equal(stats.hasErrors(), false, stats.toString({ all: false, errors: true }));
    const entry = path.join(root, 'dist/main.cjs');
    if (expectedError) {
      assert.throws(() => require(entry), (error) => {
        assert.match(error.message, expectedError);
        assert.match(error.message, /supportsSharedIdentity/);
        assert.match(error.message, /implementation/);
        return true;
      });
    } else {
      const result = require(entry);
      if (scenario === 'unused-issuer-layer' || scenario === 'reachable-child-shared') {
        assert.equal(await result(), scenario === 'unused-issuer-layer' ? 'one' : 'dependency');
      } else if (scenario === 'paired-isolated-provider') {
        await result.instance.initializeSharing('default');
        // An external unlayered host must not receive a server-only provider.
        const host = result.runtime.init({ name: 'unlayered_external_host', shared: {} });
        host.initShareScopeMap('default', result.instance.shareScopeMap.default);
        assert.equal(await host.loadShare('one', { customShareInfo: {
          scope: ['default'], shareConfig: { requiredVersion: false },
        } }), false);
      } else {
        // Full getters are observable by external hosts using the registration.
        for (const arg of result.initOptions.shared.key) {
          const name = scenario.endsWith('layers') ? arg.shareConfig.layer : arg.scope[0];
          assert.equal((await arg.get())(), name);
        }
      }
    }
  } finally {
    await new Promise((resolve, reject) => compiler.close((error) => error ? reject(error) : resolve()));
    fs.rmSync(root, { recursive: true, force: true });
  }
}
