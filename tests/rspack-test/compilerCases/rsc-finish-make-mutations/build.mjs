import assert from 'node:assert/strict';
import path from 'node:path';
import { rspack } from '@rspack/core';
import { closeCompiler, runCompiler } from '@rspack/test-tools/helper/lifecycle';
import { createFsFromVolume, Volume } from 'memfs';

const context = import.meta.dirname;

function rebuildWithLateAction() {
  let injectAction = false;
  return {
    apply(compiler) {
      compiler.hooks.compilation.tap('RebuildWithLateAction', (compilation) => {
        rspack.NormalModule.getCompilationHooks(compilation).loader.tap(
          'RebuildWithLateAction',
          (loaderContext) => {
            loaderContext.injectAction = injectAction;
          },
        );
      });
      compiler.hooks.finishMake.tapPromise(
        { name: 'RebuildWithLateAction', stage: 2 ** 31 - 1 },
        async (compilation) => {
          const outer = [...compilation.modules].find(
            (module) => module.resource === path.join(context, 'outer.js'),
          );
          assert.ok(outer, 'Outer was built from the RSC entry');
          injectAction = true;
          await new Promise((resolve, reject) => {
            compilation.rebuildModule(outer, (error) =>
              error ? reject(error) : resolve(),
            );
          });
        },
      );
    },
  };
}

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

export default async function run(scenario) {
  const { createPlugins, Layers } = rspack.experiments.rsc;
  const { ServerPlugin, ClientPlugin } = createPlugins();
  const lateAction = scenario === 'late-action';
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
      module: {
        rules: lateAction
          ? [...rules, { test: /outer\.js$/, loader: './late-action-loader.mjs' }]
          : rules,
      },
      plugins: [
        new ClientPlugin(),
        lateAction ? rebuildWithLateAction() : appendOverrides,
      ],
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
    if (lateAction) {
      assert.equal(
        Object.keys(manifest.serverManifest).length,
        2,
        'Both the initial and late actions should be registered',
      );
    } else {
      const css = outputFileSystem.readFileSync(
        path.join(clientPath, 'main.css'),
        'utf8',
      );
      assert.match(css, /color:\s*red[\s\S]*color:\s*blue/);
    }
  } finally {
    await closeCompiler(compiler);
  }
}
