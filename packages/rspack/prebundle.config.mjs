// @ts-check
import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

/** @type {import('prebundle').Config} */
export default {
  dependencies: [
    '@swc/types',
    'browserslist-load-config',
    {
      name: 'webpack-sources',
      copyDts: true,
    },
    {
      name: 'watchpack',
      dtsExternals: ['graceful-fs'],
      afterBundle(task) {
        const importStatement = "import fs from 'graceful-fs';";
        const dtsPath = join(task.distPath, 'index.d.ts');
        const content = readFileSync(dtsPath, 'utf-8');
        writeFileSync(
          dtsPath,
          content.replace(importStatement, `// @ts-ignore\n${importStatement}`),
        );
      },
    },
  ],
};
