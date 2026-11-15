import assert from 'node:assert/strict';
import path from 'node:path';
import { createRequire } from 'node:module';
import { rspack, experiments, EntryPlugin } from '@rspack/core';
import {
  runCompiler,
  closeCompiler,
} from '@rspack/test-tools/helper/lifecycle';

const require = createRequire(import.meta.url);

export default async function build(
  root,
  mode,
  value,
  translations,
  {
    readonly = false,
    dynamic = false,
    buildDependency = null,
    child = false,
    childValue = value + 100,
  } = {},
) {
  const built = [];
  const restored = [];
  const childBuilt = [];
  const childRestored = [];
  const inputs = {
    'virtual/value.js': `module.exports = ${value};`,
    ...(buildDependency === null
      ? {}
      : { 'build-config.js': `module.exports = ${buildDependency};` }),
    ...Object.fromEntries(
      Object.entries(translations).map(([name, text]) => [
        `translations/${name}.js`,
        `module.exports = ${JSON.stringify(text)};`,
      ]),
    ),
  };
  const virtual = new experiments.VirtualModulesPlugin(dynamic ? {} : inputs);
  const compiler = rspack({
    context: import.meta.dirname,
    mode,
    target: 'node',
    entry: './entry.js',
    devtool: false,
    output: {
      path: path.join(root, 'dist'),
      filename: 'bundle.cjs',
      library: { type: 'commonjs2' },
    },
    optimization: { minimize: false, concatenateModules: false },
    experiments: {
      newCache: {
        module: true,
        codeGeneration: true,
        resolver: true,
        devtool: true,
        loader: true,
        minimize: true,
      },
    },
    cache: {
      type: 'persistent',
      readonly,
      buildDependencies:
        buildDependency === null
          ? []
          : [path.join(import.meta.dirname, 'build-config.js')],
      storage: { type: 'filesystem', directory: path.join(root, 'cache') },
    },
    plugins: [
      virtual,
      {
        apply(compiler) {
          if (child) {
            compiler.hooks.make.tapPromise(
              'VirtualTimestampChild',
              async (compilation) => {
                const childCompiler = compilation.createChildCompiler(
                  'virtual-child',
                  { filename: 'child.cjs', library: { type: 'commonjs2' } },
                  [new rspack.library.EnableLibraryPlugin('commonjs2')],
                );
                const childVirtual = new experiments.VirtualModulesPlugin();
                childVirtual.apply(childCompiler);
                // Child compilers do not run environment hooks automatically.
                childCompiler.hooks.afterEnvironment.call();
                new EntryPlugin(compiler.context, './entry.js').apply(
                  childCompiler,
                );
                childCompiler.hooks.thisCompilation.tap(
                  'VirtualTimestampChild',
                  (childCompilation) => {
                    for (const [file, content] of Object.entries({
                      ...inputs,
                      'virtual/value.js': `module.exports = ${childValue};`,
                    }))
                      childVirtual.writeModule(file, content);
                    childCompilation.hooks.buildModule.tap(
                      'VirtualTimestampChild',
                      (module) => childBuilt.push(module.identifier()),
                    );
                    childCompilation.hooks.stillValidModule.tap(
                      'VirtualTimestampChild',
                      (module) => childRestored.push(module.identifier()),
                    );
                  },
                );
                await new Promise((resolve, reject) =>
                  childCompiler.runAsChild((error) =>
                    error ? reject(error) : resolve(),
                  ),
                );
              },
            );
          }
          compiler.hooks.thisCompilation.tap(
            'VirtualTimestampTest',
            (compilation) => {
              if (dynamic) {
                for (const [file, content] of Object.entries(inputs))
                  virtual.writeModule(file, content);
              }
              compilation.hooks.buildModule.tap(
                'VirtualTimestampTest',
                (module) => built.push(module.identifier()),
              );
              compilation.hooks.stillValidModule.tap(
                'VirtualTimestampTest',
                (module) => restored.push(module.identifier()),
              );
            },
          );
        },
      },
    ],
  });
  try {
    const stats = await runCompiler(compiler);
    assert.equal(stats.hasErrors(), false, stats.toString());
    const bundle = path.join(root, 'dist/bundle.cjs');
    delete require.cache[require.resolve(bundle)];
    let childOutput = null;
    if (child) {
      const childBundle = path.join(root, 'dist/child.cjs');
      delete require.cache[require.resolve(childBundle)];
      childOutput = require(childBundle);
    }
    return {
      output: require(bundle),
      built,
      restored,
      childBuilt,
      childRestored,
      childOutput,
      cache: stats.toJson({ all: false, cacheInfo: true }).cacheInfo,
    };
  } finally {
    await closeCompiler(compiler);
  }
}
