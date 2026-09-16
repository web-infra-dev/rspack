import fs from 'node:fs';
import { rspack } from '@rspack/core';

const {
  experiments: { RsdoctorPlugin },
} = rspack;

/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    concatenateModules: true,
  },
  plugins: [
    new RsdoctorPlugin({
      moduleGraphFeatures: ['graph', 'sources'],
      chunkGraphFeatures: false,
    }),
    {
      apply(compiler) {
        compiler.hooks.compilation.tap(
          'TestPlugin::ModuleIds',
          (compilation) => {
            let modules = [];
            const hooks = RsdoctorPlugin.getCompilationHooks(compilation);
            hooks.moduleGraph.tap('TestPlugin::ModuleIds', (data) => {
              modules = data.modules;
            });
            hooks.moduleSources.tap('TestPlugin::ModuleIds', (data) => {
              const { moduleOriginalSources } = data;
              expect(moduleOriginalSources.length).toBe(5);
              for (const module of modules) {
                const moduleSource = moduleOriginalSources.find(
                  (s) => s.module === module.ukey,
                );
                expect(moduleSource.source).toBe(
                  fs.readFileSync(module.path, 'utf-8'),
                );
              }
            });
          },
        );
      },
    },
  ],
};
