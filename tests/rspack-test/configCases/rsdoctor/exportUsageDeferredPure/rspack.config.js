const {
  experiments: { RsdoctorPlugin },
} = require('@rspack/core');
const path = require('path');

function normalizeRequest(request) {
  return request.replaceAll('\\', '/');
}

function hasEdge(edges, expected) {
  return edges.some((edge) =>
    Object.keys(expected).every((key) => {
      if (Array.isArray(expected[key])) {
        return (
          Array.isArray(edge[key]) &&
          edge[key].length === expected[key].length &&
          edge[key].every((item, index) => item === expected[key][index])
        );
      }
      return edge[key] === expected[key];
    }),
  );
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  optimization: {
    sideEffects: true,
    innerGraph: true,
    usedExports: true,
    concatenateModules: false,
  },
  experiments: {
    pureFunctions: true,
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
          'TestPlugin::ExportUsageDeferredPure',
          (compilation) => {
            const hooks = RsdoctorPlugin.getCompilationHooks(compilation);
            hooks.moduleGraph.tap(
              'TestPlugin::ExportUsageDeferredPure',
              (moduleGraph) => {
                moduleGraphCalled = true;
                const modulePathByUkey = new Map(
                  moduleGraph.modules.map((module) => [
                    module.ukey,
                    normalizeRequest(module.path),
                  ]),
                );
                const edges = moduleGraph.exportUsageEdges.map(
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
                );

                expect(
                  hasEdge(edges, {
                    originModulePath: normalizeRequest(
                      path.join(__dirname, 'index.js'),
                    ),
                    originExport: null,
                    targetModulePath: normalizeRequest(
                      path.join(__dirname, 're-export.js'),
                    ),
                    targetExport: ['c'],
                  }),
                ).toBe(true);
                expect(
                  hasEdge(edges, {
                    originModulePath: normalizeRequest(
                      path.join(__dirname, 're-export.js'),
                    ),
                    originExport: ['c'],
                    targetModulePath: normalizeRequest(
                      path.join(__dirname, 'dep.js'),
                    ),
                    targetExport: ['sideEffect'],
                  }),
                ).toBe(true);
                expect(
                  hasEdge(edges, {
                    originModulePath: normalizeRequest(
                      path.join(__dirname, 're-export.js'),
                    ),
                    originExport: ['b'],
                    targetModulePath: normalizeRequest(
                      path.join(__dirname, 'dep.js'),
                    ),
                    targetExport: ['sideEffect'],
                  }),
                ).toBe(true);
              },
            );
          },
        );
        compiler.hooks.done.tap('TestPlugin::ExportUsageDeferredPure', () => {
          expect(moduleGraphCalled).toBe(true);
        });
      },
    },
  ],
};
