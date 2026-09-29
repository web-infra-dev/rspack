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
    definePlugin({
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
                      path.join(import.meta.dirname, 'index.js'),
                    ),
                    originExport: null,
                    targetModulePath: normalizeRequest(
                      path.join(import.meta.dirname, 'entryA.js'),
                    ),
                    targetExport: ['EntryA'],
                  }),
                ).toBe(true);
                expect(
                  hasEdge(edges, {
                    originModulePath: normalizeRequest(
                      path.join(import.meta.dirname, 'entryA.js'),
                    ),
                    originExport: ['EntryA'],
                    targetModulePath: normalizeRequest(
                      path.join(import.meta.dirname, 'b.js'),
                    ),
                    targetExport: ['bar'],
                  }),
                ).toBe(true);
                expect(
                  hasEdge(edges, {
                    originModulePath: normalizeRequest(
                      path.join(import.meta.dirname, 'b.js'),
                    ),
                    originExport: ['bar'],
                    targetModulePath: normalizeRequest(
                      path.join(import.meta.dirname, 'c.js'),
                    ),
                    targetExport: ['baz'],
                  }),
                ).toBe(true);
                expect(
                  hasEdge(edges, {
                    originModulePath: normalizeRequest(
                      path.join(import.meta.dirname, 'b.js'),
                    ),
                    originExport: ['unusedBar'],
                    targetModulePath: normalizeRequest(
                      path.join(import.meta.dirname, 'c.js'),
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
    }),
  ],
});
