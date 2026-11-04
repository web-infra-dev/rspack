import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';
import { existsSync, mkdirSync, symlinkSync } from 'node:fs';
import { join } from 'node:path';

const linkedDirectory = join(
  import.meta.dirname,
  'node_modules/linked-importers',
);
mkdirSync(join(import.meta.dirname, 'node_modules'), { recursive: true });
if (!existsSync(linkedDirectory)) {
  symlinkSync(import.meta.dirname, linkedDirectory, 'junction');
}
const virtualImporter = join(import.meta.dirname, 'virtual-importer.js');

export default defineConfig(
  [false, true].flatMap((symlinks) =>
    [false, true].flatMap((usedExports) =>
      [false, true].map((concatenateModules) => ({
        mode: 'production' as const,
        resolve: {
          symlinks,
          alias: { './protected.js': join(linkedDirectory, 'protected.js') },
        },
        plugins: [
          new rspack.experiments.VirtualModulesPlugin({
            [virtualImporter]:
              'import "./virtual-effect.js"; export const value = 1;',
            [join(import.meta.dirname, 'virtual-effect.js')]:
              'globalThis.__directImportTrace.push("virtual");',
          }),
          new rspack.experiments.RstestPlugin({
            injectModulePathName: false,
            importMetaPathName: false,
            hoistMockModule: false,
            manualMockRoot: import.meta.dirname,
            preserveSideEffectImports: async () => [
              join(linkedDirectory, 'protected.js'),
              virtualImporter,
            ],
          }),
        ],
        optimization: {
          sideEffects: true,
          usedExports,
          concatenateModules,
          minimize: false,
        },
      })),
    ),
  ),
);
