import fs from 'node:fs';
import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import type { Compiler, WatchFileSystem } from '@rspack/core';

interface WatchpackFileSystem extends WatchFileSystem {
  watcher?: {
    watcherOptions?: {
      ignored: (entry: string) => boolean;
    };
  } | null;
}

class VerifyWindowsIgnoredGlobPlugin {
  apply(compiler: Compiler) {
    compiler.hooks.afterEnvironment.tap(
      'VerifyWindowsIgnoredGlobPlugin',
      () => {
        const wfs = compiler.watchFileSystem as WatchpackFileSystem | null;
        if (!wfs) {
          return;
        }

        const originalWatch = wfs.watch.bind(wfs);
        wfs.watch = (...args) => {
          const watcher = originalWatch(...args);
          const ignored = wfs.watcher?.watcherOptions?.ignored;
          const matchesWindowsPath = ignored?.(
            'C:/Users/Alice/project/node_modules/pkg/index.js',
          );
          fs.writeFileSync(
            path.join(compiler.outputPath, 'ignored-glob.json'),
            JSON.stringify({ matchesWindowsPath }),
          );
          return watcher;
        };
      },
    );
  }
}

export default defineConfig({
  plugins: [new VerifyWindowsIgnoredGlobPlugin()],
  watchOptions: {
    ignored: String.raw`C:\Users\Alice\project\node_modules`,
  },
});
