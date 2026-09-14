const fs = require('node:fs');
const path = require('node:path');
const rspack = require('@rspack/core');

let issuerBuilds = 0;
let issuerCacheHits = 0;
let targetFactorizations = 0;

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
  description:
    'should invalidate a cached URL issuer when its factory probe resolves to a different module type',
  options(context) {
    issuerBuilds = 0;
    issuerCacheHits = 0;
    targetFactorizations = 0;
    const root = context.getDist('src');
    fs.mkdirSync(root, { recursive: true });
    fs.writeFileSync(
      path.join(root, 'index.js'),
      'module.exports = new URL("./target", import.meta.url);',
    );
    fs.writeFileSync(
      path.join(root, 'target.js'),
      'globalThis.URL_TYPE_PROBE_EXECUTED = true;',
    );
    fs.writeFileSync(path.join(root, 'target.txt'), 'asset fallback');
    // Stable timestamps ensure the unchanged warm run can hit the module cache.
    const timestamp = new Date(Date.now() - 20000);
    for (const name of ['index.js', 'target.js', 'target.txt']) {
      fs.utimesSync(path.join(root, name), timestamp, timestamp);
    }
    return {
      context: root,
      entry: './index.js',
      mode: 'development',
      devtool: false,
      target: 'web',
      incremental: false,
      cache: true,
      experiments: {
        newCache: { module: true, loader: false, codeGeneration: false },
      },
      resolve: { byDependency: { url: { extensions: ['.js', '.txt'] } } },
      module: {
        rules: [
          { test: /target\.js$/, dependency: 'url', type: 'javascript/auto' },
        ],
      },
      output: {
        path: context.getDist('output'),
        filename: 'bundle.js',
        chunkFilename: 'url-[id].js',
        assetModuleFilename: '[name][ext]',
        publicPath: '/assets/',
      },
      plugins: [
        (compiler) => {
          compiler.hooks.compilation.tap(
            'CheckUrlProbeCache',
            (compilation, { normalModuleFactory }) => {
              normalModuleFactory.hooks.beforeResolve.tap(
                'CheckUrlProbeCache',
                ({ request }) => {
                  if (request === './target') targetFactorizations++;
                },
              );
              compilation.hooks.buildModule.tap(
                'CheckUrlProbeCache',
                (module) => {
                  if (module.rawRequest === './index.js') issuerBuilds++;
                },
              );
              compilation.hooks.stillValidModule.tap(
                'CheckUrlProbeCache',
                (module) => {
                  if (module.rawRequest === './index.js') issuerCacheHits++;
                },
              );
            },
          );
        },
      ],
    };
  },
  async build(context, initialCompiler) {
    const compilers = [];
    try {
      for (let step = 0; step < 4; step++) {
        const target = context.getDist('src/target.js');
        if (step === 2) fs.unlinkSync(target);
        if (step === 3)
          fs.writeFileSync(
            target,
            'globalThis.URL_TYPE_PROBE_EXECUTED = "restored";',
          );

        // Each fresh compiler takes the ordinary build path, while sharing only
        // the in-memory Cache. No incremental graph or filesystem cache is reused.
        const compiler =
          step === 0
            ? initialCompiler
            : rspack(context.getCompiler().getOptions());
        if (step > 0) {
          compiler.cache = initialCompiler.cache;
          compiler.outputFileSystem = initialCompiler.outputFileSystem;
          compilers.push(compiler);
        }
        const stats = await new Promise((resolve, reject) => {
          compiler.run((error, stats) =>
            error ? reject(error) : resolve(stats),
          );
        });
        expect(stats.hasErrors()).toBe(false);
        expect(targetFactorizations).toBe(step + 1);
        const compilation = stats.compilation;
        const issuer = [...compilation.modules].find(
          (module) => module.rawRequest === './index.js',
        );
        const isAsset = step === 2;
        expect(issuer.blocks).toHaveLength(isAsset ? 0 : 1);
        const dependency = isAsset
          ? issuer.dependencies.find((dep) => dep.type === 'new URL()')
          : issuer.blocks[0].dependencies[0];
        expect(compilation.moduleGraph.getModule(dependency).type).toBe(
          isAsset ? 'asset/resource' : 'javascript/auto',
        );
        expect(issuerBuilds).toBe(step === 0 ? 1 : step);
        expect(issuerCacheHits).toBe(step === 0 ? 0 : 1);
        if (isAsset) {
          expect(
            compilation.getAsset('target.txt').source.source().toString(),
          ).toBe('asset fallback');
        }
      }
    } finally {
      for (const compiler of compilers) {
        await new Promise((resolve, reject) =>
          compiler.close((error) => (error ? reject(error) : resolve())),
        );
      }
    }
  },
};
