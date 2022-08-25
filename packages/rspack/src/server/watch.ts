import path from 'path';
import chokidar from 'chokidar';
import * as Hmr from './hmr';
import type { WatchOptions, FSWatcher } from 'chokidar';
import type { ResolvedDevConfig } from '.';

export function createWatcher(config: ResolvedDevConfig): FSWatcher {

  const watchOptions: WatchOptions = {
    ignored: [
      '**/node_modules/**',
      '**/.git/**',
      "**/dist/**",
      "**/lib/**"
    ],
  };

  const watcher = chokidar.watch(path.resolve(config.context ?? process.cwd()), watchOptions);
  return watcher;
}

