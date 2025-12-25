import type EventEmitter from 'node:events';
import path from 'node:path';
import type { VirtualFileStore } from '@rspack/binding';
import type { Compiler } from './Compiler';
import NativeWatchFileSystem from './NativeWatchFileSystem';
import type { WatchFileSystem } from './util/fs';

const PLUGIN_NAME = 'VirtualModulesPlugin';

const VFILES_BY_COMPILER = new WeakMap<Compiler, Record<string, string>>();

export class VirtualModulesPlugin {
  #staticModules: Record<string, string> | null;
  #compiler?: Compiler;

  #store?: VirtualFileStore;

  constructor(modules?: Record<string, string>) {
    this.#staticModules = modules || null;
  }

  apply(compiler: Compiler) {
    this.#compiler = compiler;

    compiler.hooks.afterEnvironment.tap(PLUGIN_NAME, () => {
      const record = VFILES_BY_COMPILER.get(compiler) || {};

      if (this.#staticModules) {
        for (const [filePath, content] of Object.entries(this.#staticModules)) {
          const fullPath = path.resolve(compiler.context, filePath);
          record[fullPath] = content;
        }
      }

      VFILES_BY_COMPILER.set(compiler, record);
    });
  }

  public writeModule(filePath: string, contents: string): void {
    if (!this.#compiler) {
      throw new Error('Plugin has not been initialized');
    }

    const store = this.getVirtualFileStore();

    const fullPath = path.resolve(this.#compiler.context, filePath);

    store.writeVirtualFileSync(fullPath, contents);

    notifyWatchers(this.#compiler, fullPath, Date.now());
  }

  private getVirtualFileStore() {
    if (this.#store) return this.#store;

    const store = this.#compiler?.__internal__get_virtual_file_store();
    if (!store) {
      throw new Error('Virtual file store has not been initialized');
    }
    this.#store = store;

    return store;
  }

  static __internal__take_virtual_files(compiler: Compiler) {
    const record = VFILES_BY_COMPILER.get(compiler);
    if (record) {
      VFILES_BY_COMPILER.delete(compiler);
      return Object.entries(record).map(([path, content]) => ({
        path,
        content,
      }));
    }
  }
}

function notifyWatchers(compiler: Compiler, fullPath: string, time: number) {
  if (compiler.watchFileSystem instanceof NativeWatchFileSystem) {
    compiler.watchFileSystem.triggerEvent('change', fullPath);
  } else {
    notifyJsWatchers(compiler, fullPath, time);
  }
}

function notifyJsWatchers(compiler: Compiler, fullPath: string, time: number) {
  if (
    compiler.watchFileSystem &&
    isNodeWatchFileSystem(compiler.watchFileSystem)
  ) {
    const watcher = compiler.watchFileSystem.watcher;
    if (!watcher) return;
    const fileWatcher = watcher.fileWatchers.get(fullPath);
    if (fileWatcher) {
      fileWatcher.watcher.emit('change', time, null);
    }
  }
}

// rspack uses a precompiled watchpack, of which the implementation doesn't match the .d.ts
interface Watcher extends EventEmitter {
  path: string;
}
interface WatchpacFileWatcher {
  watcher: Watcher;
}
interface Watchpack {
  fileWatchers: Map<string, WatchpacFileWatcher>;
}
interface NodeWatchFileSystem extends WatchFileSystem {
  watcher?: Watchpack;
}

function isNodeWatchFileSystem(fs: WatchFileSystem): fs is NodeWatchFileSystem {
  return 'watch' in fs;
}
