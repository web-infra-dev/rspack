import assert from 'node:assert/strict';
import path from 'node:path';
import { rspack } from '@rspack/core';
import { closeCompiler, runCompiler } from '@rspack/test-tools/helper/lifecycle';
import { createFsFromVolume, Volume } from 'memfs';

const context = import.meta.dirname;

const appendOverrides = {
  apply(compiler) {
    compiler.hooks.finishMake.tapPromise('AppendOverrides', (compilation) =>
      new Promise((resolve, reject) => {
        compilation.addEntry(
          compiler.context,
          rspack.EntryPlugin.createDependency('./overrides.css'),
          { name: 'main' },
          (error) => (error ? reject(error) : resolve()),
        );
      }),
    );
  },
};

export default async function run() {
  const { createPlugins, Layers } = rspack.experiments.rsc;
  const { ServerPlugin, ClientPlugin } = createPlugins();
  const clientPath = path.join(context, 'dist', 'client');
  const outputFileSystem = createFsFromVolume(new Volume());
  let manifest;

  const rules = [
    {
      test: /\.js$/,
      loader: 'builtin:swc-loader',
      options: { rspackExperiments: { reactServerComponents: true } },
    },
    { test: /\.css$/, type: 'css' },
  ];
  const compiler = rspack([
    {
      context,
      mode: 'production',
      target: 'node',
      entry: { main: { import: './server.js', layer: Layers.rsc } },
      output: { path: path.join(context, 'dist', 'server') },
      module: {
        rules: [
          ...rules,
          {
            issuerLayer: Layers.rsc,
            resolve: { conditionNames: ['react-server', '...'] },
          },
        ],
      },
      plugins: [
        new ServerPlugin({
          onManifest(result) {
            manifest = result.main;
          },
        }),
      ],
    },
    {
      context,
      mode: 'production',
      target: 'web',
      entry: './client.js',
      output: { path: clientPath },
      optimization: { minimize: false },
      module: { rules },
      plugins: [new ClientPlugin(), appendOverrides],
    },
  ]);
  compiler.outputFileSystem = outputFileSystem;
  try {
    const stats = await runCompiler(compiler);
    assert.ok(!stats.hasErrors(), stats.toString('errors-only'));
    assert.deepEqual(
      Object.keys(manifest.clientManifest)
        .map((file) => path.basename(file))
        .sort(),
      ['inner.js', 'outer.js'],
    );
    const css = outputFileSystem.readFileSync(
      path.join(clientPath, 'main.css'),
      'utf8',
    );
    assert.match(css, /color:\s*red[\s\S]*color:\s*blue/);
  } finally {
    await closeCompiler(compiler);
  }
}
