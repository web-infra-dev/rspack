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
      afterBundle(task) {
        // Suppress missing-module errors for optional Hono peer type imports in generated d.ts files.
        replaceFileContent(join(task.distPath, 'index.d.ts'), (content) => {
          return content
            .replace(
              `import { HttpBindings } from '@hono/node-server';`,
              `// @ts-ignore
import { HttpBindings } from '@hono/node-server';`,
            )
            .replace(
              `import { MiddlewareHandler } from 'hono';`,
              `// @ts-ignore
import { MiddlewareHandler } from 'hono';`,
            );
        });
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
        const ignoredImportStatement = `// @ts-ignore\n${importStatement}`;
        const dtsPath = join(task.distPath, 'index.d.ts');
        replaceFileContent(
          dtsPath,
          (content) =>
            `${content.replace(importStatement, ignoredImportStatement)}
export type WatchOptions = Watchpack.WatchOptions;
`,
        );
      },
    },
  ],
};
