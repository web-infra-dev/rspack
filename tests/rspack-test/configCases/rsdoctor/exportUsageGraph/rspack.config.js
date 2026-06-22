const {
  experiments: { RsdoctorPlugin },
} = require('@rspack/core');
const path = require('path');

function normalizeRequest(request) {
  return request.replaceAll('\\', '/');
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  optimization: {
    concatenateModules: false,
    usedExports: true,
  },
  output: {
    assetModuleFilename: '[path][name][ext]',
  },
  module: {
    parser: {
      javascript: {
        strictThisContextOnImports: true,
        reexportExportsPresence: false,
      },
    },
  },
  plugins: [
    new RsdoctorPlugin({
      moduleGraphFeatures: ['graph'],
      chunkGraphFeatures: false,
      exportUsageGraph: true,
    }),
    {
      apply(compiler) {
        let moduleGraphCalled = false;
        compiler.hooks.compilation.tap(
          'TestPlugin::ExportUsageGraph',
          (compilation) => {
            const hooks = RsdoctorPlugin.getCompilationHooks(compilation);
            hooks.moduleGraph.tap(
              'TestPlugin::ExportUsageGraph',
              (moduleGraph) => {
                moduleGraphCalled = true;
                const modulePathByUkey = new Map(
                  moduleGraph.modules.map((module) => [
                    module.ukey,
                    normalizeRequest(module.path),
                  ]),
                );
                const edges = moduleGraph.exportUsageEdges
                  .map(
                    ([
                      originModule,
                      originExport,
                      targetModule,
                      targetExport,
                    ]) => ({
                      originModulePath: modulePathByUkey.get(originModule),
                      originExport,
                      targetModulePath: modulePathByUkey.get(targetModule),
                      targetExport,
                    }),
                  )
                  .sort((a, b) =>
                    `${a.originModulePath}:${a.originExport}:${a.targetModulePath}:${a.targetExport}` >
                    `${b.originModulePath}:${b.originExport}:${b.targetModulePath}:${b.targetExport}`
                      ? 1
                      : -1,
                  );
                expect(edges).toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'index.js'),
                  ),
                  originExport: null,
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'lib.js'),
                  ),
                  targetExport: ['foo'],
                });
                expect(edges).toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'lib.js'),
                  ),
                  originExport: ['foo'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'shared.js'),
                  ),
                  targetExport: ['bar'],
                });
                expect(edges).toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'lib.js'),
                  ),
                  originExport: ['foo'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'shared.js'),
                  ),
                  targetExport: ['namespaceFoo'],
                });
                expect(edges).toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'lib.js'),
                  ),
                  originExport: ['foo'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'star-a.js'),
                  ),
                  targetExport: ['multiFoo'],
                });
                expect(edges).toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'lib.js'),
                  ),
                  originExport: ['foo'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'star-b.js'),
                  ),
                  targetExport: ['multiBar'],
                });
                expect(edges).toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'index.js'),
                  ),
                  originExport: null,
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'json-user.js'),
                  ),
                  targetExport: ['getJsonName'],
                });
                expect(edges).toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'json-user.js'),
                  ),
                  originExport: ['getJsonName'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'data.json'),
                  ),
                  targetExport: ['name'],
                });
                expect(edges).toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'cjs-user.js'),
                  ),
                  originExport: ['getCjsFoo'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'cjs.js'),
                  ),
                  targetExport: null,
                });
                expect(edges).toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'callable-user.js'),
                  ),
                  originExport: ['callMethod'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'callable-source.js'),
                  ),
                  targetExport: ['callableObj'],
                });
                expect(edges).toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'namespace-user.js'),
                  ),
                  originExport: ['getNamespaceValue'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'namespace-source.js'),
                  ),
                  targetExport: ['nsValue'],
                });
                expect(edges).toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'namespace-whole-user.js'),
                  ),
                  originExport: ['getNamespaceObject'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'namespace-source.js'),
                  ),
                  targetExport: null,
                });
                expect(edges).toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'normal-user.js'),
                  ),
                  originExport: ['getNormalReexportUsed'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'normal-source.js'),
                  ),
                  targetExport: ['obj', 'used'],
                });
                expect(edges).toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'overlap-star-user.js'),
                  ),
                  originExport: ['getOverlapStar'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'overlap-star-a.js'),
                  ),
                  targetExport: ['overlapFoo'],
                });
                expect(edges).toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'overlap-star-user.js'),
                  ),
                  originExport: ['getOverlapStar'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'overlap-star-b.js'),
                  ),
                  targetExport: ['overlapBar'],
                });
                expect(edges).toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'star-nested-user.js'),
                  ),
                  originExport: ['getStarNestedUsed'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'star-nested-source.js'),
                  ),
                  targetExport: ['nestedObj', 'used'],
                });
                expect(edges).toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'url-user.js'),
                  ),
                  originExport: ['getAssetUrl'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'url-asset.wasm'),
                  ),
                  targetExport: null,
                });
                expect(edges).not.toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'json-user.js'),
                  ),
                  originExport: ['getJsonName'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'data.json'),
                  ),
                  targetExport: ['default', 'name'],
                });
                expect(edges).not.toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'cjs-user.js'),
                  ),
                  originExport: ['getCjsFoo'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'cjs.js'),
                  ),
                  targetExport: ['default', 'foo'],
                });
                expect(edges).not.toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'callable-user.js'),
                  ),
                  originExport: ['callMethod'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'callable-source.js'),
                  ),
                  targetExport: ['callableObj', 'method'],
                });
                expect(edges).not.toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'empty-destructure-user.js'),
                  ),
                  originExport: ['getEmptyDestructure'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'empty-destructure-source.js'),
                  ),
                  targetExport: ['emptyDestructureValue'],
                });
                expect(edges).not.toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'empty-destructure-user.js'),
                  ),
                  originExport: ['getEmptyDestructure'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'empty-destructure-source.js'),
                  ),
                  targetExport: null,
                });
                expect(edges).not.toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'namespace-user.js'),
                  ),
                  originExport: ['getNamespaceValue'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'namespace-source.js'),
                  ),
                  targetExport: null,
                });
                expect(edges).not.toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'namespace-whole-user.js'),
                  ),
                  originExport: ['getNamespaceObject'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'namespace-source.js'),
                  ),
                  targetExport: [],
                });
                expect(edges).not.toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'normal-user.js'),
                  ),
                  originExport: ['getNormalReexportUsed'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'normal-source.js'),
                  ),
                  targetExport: ['obj'],
                });
                expect(edges).not.toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'overlap-star-user.js'),
                  ),
                  originExport: ['getOverlapStar'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'overlap-star-b.js'),
                  ),
                  targetExport: ['overlapFoo'],
                });
                expect(edges).not.toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'star-nested-user.js'),
                  ),
                  originExport: ['getStarNestedUsed'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'star-nested-source.js'),
                  ),
                  targetExport: ['nestedObj'],
                });
                expect(edges).not.toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'undefined-barrel.js'),
                  ),
                  originExport: ['jsonNamed'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'undefined-json.json'),
                  ),
                  targetExport: ['foo'],
                });
                expect(edges).not.toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'lib.js'),
                  ),
                  originExport: ['foo'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'star-b.js'),
                  ),
                  targetExport: ['multiFoo'],
                });
                expect(edges).not.toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'lib.js'),
                  ),
                  originExport: ['foo'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'star-a.js'),
                  ),
                  targetExport: ['multiBar'],
                });
                expect(edges).not.toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'lib.js'),
                  ),
                  originExport: ['foo'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'shared.js'),
                  ),
                  targetExport: null,
                });
                expect(edges).not.toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'lib.js'),
                  ),
                  originExport: ['unusedFoo'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'shared.js'),
                  ),
                  targetExport: ['bar'],
                });
                expect(edges).not.toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'barrel.js'),
                  ),
                  originExport: ['unusedReexport'],
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'shared.js'),
                  ),
                  targetExport: ['unused'],
                });
                expect(edges).not.toContainEqual({
                  originModulePath: normalizeRequest(
                    path.join(__dirname, 'star.js'),
                  ),
                  originExport: null,
                  targetModulePath: normalizeRequest(
                    path.join(__dirname, 'shared.js'),
                  ),
                  targetExport: null,
                });
              },
            );
          },
        );
        compiler.hooks.done.tap('TestPlugin::ExportUsageGraph', () => {
          expect(moduleGraphCalled).toBe(true);
        });
      },
    },
  ],
};
