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
  entry: './index.js',
  optimization: {
    concatenateModules: false,
    usedExports: true,
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
          'TestPlugin::ExportUsageClassGraph',
          (compilation) => {
            const hooks = RsdoctorPlugin.getCompilationHooks(compilation);
            hooks.moduleGraph.tap(
              'TestPlugin::ExportUsageClassGraph',
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
                      path.join(__dirname, 'entryA.js'),
                    ),
                    targetExport: ['EntryA'],
                  }),
                ).toBe(true);
                expect(
                  hasEdge(edges, {
                    originModulePath: normalizeRequest(
                      path.join(__dirname, 'entryA.js'),
                    ),
                    originExport: ['EntryA'],
                    targetModulePath: normalizeRequest(
                      path.join(__dirname, 'b.js'),
                    ),
                    targetExport: ['bar'],
                  }),
                ).toBe(true);
                expect(
                  hasEdge(edges, {
                    originModulePath: normalizeRequest(
                      path.join(__dirname, 'b.js'),
                    ),
                    originExport: ['bar'],
                    targetModulePath: normalizeRequest(
                      path.join(__dirname, 'c.js'),
                    ),
                    targetExport: ['baz'],
                  }),
                ).toBe(true);
                expect(
                  hasEdge(edges, {
                    originModulePath: normalizeRequest(
                      path.join(__dirname, 'b.js'),
                    ),
                    originExport: ['unusedBar'],
                    targetModulePath: normalizeRequest(
                      path.join(__dirname, 'c.js'),
                    ),
                    targetExport: ['unusedBaz'],
                  }),
                ).toBe(false);
              },
            );
          },
        );
        compiler.hooks.done.tap('TestPlugin::ExportUsageClassGraph', () => {
          expect(moduleGraphCalled).toBe(true);
        });
      },
    },
  ],
};
