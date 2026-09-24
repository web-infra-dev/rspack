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

        // Windows path separator normalization patch.
        // Upstream watchpack's `withoutCase` only lowercases; on Windows the
        // `DirectoryWatcher.doScan` initial-missing calculation registers
        // watcher keys with whichever separator the caller passed in and then
        // tries to delete entries keyed by `path.join()` output (backslash).
        // When rspack hands over a mix of forward-slash paths (persistent
        // cache rehydrate) and backslash paths (native fs / loader
        // addDependency), the delete pass misses every forward-slash-keyed
        // watcher and each fires a spurious `initial-missing` on hot start,
        // which cascades into `compiler.removedFiles` and a full rebuild.
        // Normalizing to forward slashes inside `withoutCase` collapses the
        // two flavors to a single map key, matching the behavior POSIX
        // already gets for free.
        const indexJsPath = join(task.distPath, 'index.js');
        replaceFileContent(indexJsPath, (content) =>
          content.replace(
            'function withoutCase(str) {\n\treturn str.toLowerCase();\n}',
            "function withoutCase(str) {\n\t// Normalize backslashes in addition to lowercasing so DirectoryWatcher.doScan's\n\t// missingWatchers map key (registered with the caller's separator) matches\n\t// path.join()'s native-separator output on Windows.\n\treturn str.toLowerCase().replace(/\\\\/g, \"/\");\n}",
          ),
        );
      },
    },
  ],
};
