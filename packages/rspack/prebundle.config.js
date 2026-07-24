import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

function replaceFileContent(filePath, replaceFn) {
  const content = readFileSync(filePath, 'utf-8');
  const newContent = replaceFn(content);
  if (newContent !== content) {
    writeFileSync(filePath, newContent);
  }
}

/** @type {import('prebundle').Config} */
export default {
  dependencies: [
    '@swc/types',
    {
      name: 'webpack-sources',
      copyDts: true,
    },
    {
      name: 'connect-next',
      dtsOnly: true,
    },
    {
      name: '@rspack/lite-tapable',
      copyDts: true,
      dtsOnly: true,
    },
    {
      name: 'http-proxy-middleware',
      dtsOnly: true,
    },
    {
      name: 'open',
      dtsOnly: true,
    },
    {
      name: 'watchpack',
      copyDts: true,
      afterBundle(task) {
        // Keep the public declaration entry at the package root. watchpack's
        // copied declarations use extensionless relative imports, which leak
        // into Rspack's generated d.ts and fail NodeNext type tests.
        const dtsPath = join(task.distPath, 'index.d.ts');
        writeFileSync(
          dtsPath,
          `import Watchpack = require("./types/index");
export default Watchpack;
export type WatchOptions = Watchpack.WatchOptions;
`,
        );

        const packageJsonPath = join(task.distPath, 'package.json');
        replaceFileContent(packageJsonPath, (content) => {
          const packageJson = JSON.parse(content);
          packageJson.types = 'index.d.ts';
          return `${JSON.stringify(packageJson, null, 2)}\n`;
        });
      },
    },
  ],
};
