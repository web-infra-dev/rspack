import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

const {
  experiments: { RsdoctorPlugin },
} = rspack;

function normalizeRequest(request: string) {
  return request.replaceAll('\\', '/');
}

type ExportUsageEdge = {
  originModulePath: string | undefined;
  originExport: string[] | null;
  targetModulePath: string | undefined;
  targetExport: string[] | null;
};

function hasEdge(edges: ExportUsageEdge[], expected: Partial<ExportUsageEdge>) {
  return edges.some((edge) =>
    (Object.keys(expected) as (keyof ExportUsageEdge)[]).every((key) => {
      const expectedValue = expected[key];
      const actualValue = edge[key];
      if (Array.isArray(expectedValue)) {
        return (
          Array.isArray(actualValue) &&
          actualValue.length === expectedValue.length &&
          actualValue.every((item, index) => item === expectedValue[index])
        );
      }
      return actualValue === expectedValue;
    }),
  );
}

export default defineConfig({
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
    definePlugin({
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
                      path.join(import.meta.dirname, 'index.js'),
                    ),
                    originExport: null,
                    targetModulePath: normalizeRequest(
                      path.join(import.meta.dirname, 're-export.js'),
                    ),
                    targetExport: ['c'],
                  }),
                ).toBe(true);
                expect(
                  hasEdge(edges, {
                    originModulePath: normalizeRequest(
                      path.join(import.meta.dirname, 're-export.js'),
                    ),
                    originExport: ['c'],
                    targetModulePath: normalizeRequest(
                      path.join(import.meta.dirname, 'dep.js'),
                    ),
                    targetExport: ['sideEffect'],
                  }),
                ).toBe(true);
                expect(
                  hasEdge(edges, {
                    originModulePath: normalizeRequest(
                      path.join(import.meta.dirname, 're-export.js'),
                    ),
                    originExport: ['b'],
                    targetModulePath: normalizeRequest(
                      path.join(import.meta.dirname, 'dep.js'),
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
    }),
  ],
});
