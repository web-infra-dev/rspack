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
      beforeBundle(task) {
        replaceFileContent(join(task.depPath, 'dist/types.d.ts'), (content) =>
          content.replace(
            "import type * as httpProxy from 'http-proxy'",
            "import type httpProxy from 'http-proxy'",
          ),
        );
      },
    },
    {
      name: 'open',
      dtsOnly: true,
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
