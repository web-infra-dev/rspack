const path = require('path');
const {
  container: { ModuleFederationPlugin },
} = require('@rspack/core');

const legacyImplementation =
  require.resolve('@module-federation/runtime-tools');
const pairedImplementation =
  require.resolve('@module-federation/runtime-tools-layers');
const { bundlerRuntime } = require(
  require.resolve('@module-federation/webpack-bundler-runtime', {
    paths: [pairedImplementation],
  }),
);
if (bundlerRuntime.supportsSharedIdentity !== true) {
  throw new Error(
    'paired-* scenarios require a runtime exporting bundlerRuntime.supportsSharedIdentity, not a marker patched on at startup',
  );
}

const treeShaking = { mode: 'runtime-infer' };
const remoteOnly = { import: false, requiredVersion: false };
const pair = (scenario) => {
  const layers = scenario.endsWith('layers');
  return Object.fromEntries(
    ['one', 'two'].map((name, index) => [
      name,
      {
        shareKey: 'key',
        requiredVersion: false,
        treeShaking,
        version: scenario === 'safe-scopes' && index ? '2.0.0' : '1.0.0',
        shareScope: layers ? 'default' : name,
        layer: layers ? name : undefined,
      },
    ]),
  );
};
const one = { one: { requiredVersion: false, treeShaking } };
const scenarios = {
  'unused-issuer-layer': [
    './lazy-one.js',
    { ...one, 'remote-only': { ...remoteOnly, issuerLayer: 'server' } },
  ],
  'reachable-child-shared': [
    './lazy-parent.js',
    {
      parent: { requiredVersion: false, treeShaking },
      dependency: { requiredVersion: false },
      'remote-only': { ...remoteOnly, issuerLayer: 'server' },
    },
  ],
  'consumer-layer': [
    './consumer.js',
    { 'remote-only': { ...remoteOnly, layer: 'server' } },
  ],
  'provider-layer': [
    './provider.js',
    { one: { layer: 'server', requiredVersion: false } },
  ],
  'fallback-scopes': ['./pair.js', pair('fallback-scopes')],
  'fallback-layers': ['./pair.js', pair('fallback-layers')],
  'safe-scopes': ['./pair.js', pair('safe-scopes')],
  'paired-scopes': ['./pair.js', pair('paired-scopes')],
  'paired-layers': ['./pair.js', pair('paired-layers')],
  'paired-isolated-provider': [
    './provider.js',
    { one: { layer: 'server', requiredVersion: false } },
  ],
};

module.exports = (env, { testPath }) => [
  // bundle0.js holds the assertions and loads each scenario bundle on demand.
  { entry: './index.js' },
  ...Object.entries(scenarios).map(([scenario, [entry, shared]]) => ({
    entry,
    mode: 'development',
    optimization: { concatenateModules: false },
    output: {
      path: path.join(testPath, scenario),
      filename: 'main.js',
      // Use forward slashes. The runtime writes publicPath into a string
      // unescaped, so a Windows path such as `...\unused-issuer-layer` would
      // be an invalid `\u` escape.
      publicPath: `${path.join(testPath, scenario).replaceAll('\\', '/')}/`,
      library: { type: 'commonjs-module' },
    },
    plugins: [
      new ModuleFederationPlugin({
        name: `identity_${scenario.replaceAll('-', '_')}`,
        implementation: scenario.startsWith('paired-')
          ? pairedImplementation
          : legacyImplementation,
        shared,
        library: { type: 'commonjs-module' },
        runtimePlugins: [path.join(__dirname, 'load-entry.cjs')],
      }),
    ],
  })),
];
