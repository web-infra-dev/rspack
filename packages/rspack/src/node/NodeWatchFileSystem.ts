/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/node/NodeWatchFileSystem.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import { EventEmitter } from 'node:events';
import { createRequire } from 'node:module';
import util from 'node:util';
import type Watchpack from 'watchpack';

import type {
  FileSystemInfoEntry,
  InputFileSystem,
  Watcher,
  WatchFileSystem,
} from '../util/fs';

const require = createRequire(import.meta.url);

type WatchpackInstance = InstanceType<typeof Watchpack>;

export default class NodeWatchFileSystem implements WatchFileSystem {
  inputFileSystem: InputFileSystem;
  watcherOptions: Watchpack.WatchOptions;
  watcher?: WatchpackInstance;
  // Long-lived emitter backing the `on`/`once` API. `watch()` replaces
  // `this.watcher` with a fresh Watchpack each cycle, so listeners must live
  // here (and be fed from each new watcher) to survive across cycles.
  #events = new EventEmitter();

  constructor(inputFileSystem: InputFileSystem) {
    this.inputFileSystem = inputFileSystem;
    this.watcherOptions = {
      aggregateTimeout: 0,
    };
  }

  watch(
    files: Iterable<string>,
    directories: Iterable<string>,
    missing: Iterable<string>,
    startTime: number,
    options: Watchpack.WatchOptions,
    callback: (
      error: Error | null,
      fileTimeInfoEntries: Map<string, FileSystemInfoEntry | 'ignore'>,
      contextTimeInfoEntries: Map<string, FileSystemInfoEntry | 'ignore'>,
      changedFiles: Set<string>,
      removedFiles: Set<string>,
    ) => void,
    callbackUndelayed: (fileName: string, changeTime: number) => void,
  ): Watcher {
    if (!files || typeof files[Symbol.iterator] !== 'function') {
      throw new Error("Invalid arguments: 'files'");
    }
    if (!directories || typeof directories[Symbol.iterator] !== 'function') {
      throw new Error("Invalid arguments: 'directories'");
    }
    if (!missing || typeof missing[Symbol.iterator] !== 'function') {
      throw new Error("Invalid arguments: 'missing'");
    }
    if (typeof callback !== 'function') {
      throw new Error("Invalid arguments: 'callback'");
    }
    if (typeof startTime !== 'number' && startTime) {
      throw new Error("Invalid arguments: 'startTime'");
    }
    if (typeof options !== 'object') {
      throw new Error("Invalid arguments: 'options'");
    }
    if (typeof callbackUndelayed !== 'function' && callbackUndelayed) {
      throw new Error("Invalid arguments: 'callbackUndelayed'");
    }

    const oldWatcher = this.watcher;
    const Watchpack = require('../compiled/watchpack/index.js');
    this.watcher = new Watchpack(options);

    if (callbackUndelayed) {
      this.watcher?.once('change', callbackUndelayed);
    }

    // Forward this cycle's watchpack events to the long-lived emitter so
    // `on`/`once` listeners keep working after the watcher is replaced.
    this.watcher?.on('change', (filename, mtime) => {
      this.#events.emit('change', filename, mtime);
    });
    this.watcher?.on('remove', (filename) => {
      this.#events.emit('remove', filename);
    });
    this.watcher?.on('aggregated', (changes, removals) => {
      this.#events.emit('aggregated', changes, removals);
    });

    const fetchTimeInfo = () => {
      const fileTimeInfoEntries = new Map();
      const contextTimeInfoEntries = new Map();
      this.watcher?.collectTimeInfoEntries(
        fileTimeInfoEntries,
        contextTimeInfoEntries,
      );
      return { fileTimeInfoEntries, contextTimeInfoEntries };
    };
    this.watcher?.once('aggregated', (changes, removals) => {
      // pause emitting events (avoids clearing aggregated changes and removals on timeout)
      this.watcher?.pause();

      if (this.inputFileSystem?.purge) {
        const fs = this.inputFileSystem;
        for (const item of changes) {
          fs.purge?.(item);
        }
        for (const item of removals) {
          fs.purge?.(item);
        }
      }
      const { fileTimeInfoEntries, contextTimeInfoEntries } = fetchTimeInfo();

      callback(
        null,
        fileTimeInfoEntries,
        contextTimeInfoEntries,
        changes,
        removals,
      );
    });

    this.watcher?.watch({ files, directories, missing, startTime });

    if (oldWatcher) {
      oldWatcher.close();
    }
    return {
      close: () => {
        if (this.watcher) {
          this.watcher.close();
          this.watcher = null as any;
        }
      },
      pause: () => {
        if (this.watcher) {
          this.watcher.pause();
        }
      },
      getAggregatedRemovals: util.deprecate(
        () => {
          const items = this.watcher?.aggregatedRemovals;
          if (items && this.inputFileSystem?.purge) {
            const fs = this.inputFileSystem;
            for (const item of items) {
              fs.purge?.(item);
            }
          }
          return items ?? new Set();
        },
        "Watcher.getAggregatedRemovals is deprecated in favor of Watcher.getInfo since that's more performant.",
        'DEP_WEBPACK_WATCHER_GET_AGGREGATED_REMOVALS',
      ),
      getAggregatedChanges: util.deprecate(
        () => {
          const items = this.watcher?.aggregatedChanges;
          if (items && this.inputFileSystem?.purge) {
            const fs = this.inputFileSystem;
            for (const item of items) {
              fs.purge?.(item);
            }
          }
          return items ?? new Set();
        },
        "Watcher.getAggregatedChanges is deprecated in favor of Watcher.getInfo since that's more performant.",
        'DEP_WEBPACK_WATCHER_GET_AGGREGATED_CHANGES',
      ),
      getFileTimeInfoEntries: util.deprecate(
        () => {
          return fetchTimeInfo().fileTimeInfoEntries;
        },
        "Watcher.getFileTimeInfoEntries is deprecated in favor of Watcher.getInfo since that's more performant.",
        'DEP_WEBPACK_WATCHER_FILE_TIME_INFO_ENTRIES',
      ),
      getContextTimeInfoEntries: util.deprecate(
        () => {
          return fetchTimeInfo().contextTimeInfoEntries;
        },
        "Watcher.getContextTimeInfoEntries is deprecated in favor of Watcher.getInfo since that's more performant.",
        'DEP_WEBPACK_WATCHER_CONTEXT_TIME_INFO_ENTRIES',
      ),
      getInfo: () => {
        const removals = this.watcher?.aggregatedRemovals ?? new Set();
        const changes = this.watcher?.aggregatedChanges ?? new Set();
        if (this.inputFileSystem?.purge) {
          const fs = this.inputFileSystem;
          if (removals) {
            for (const item of removals) {
              fs.purge?.(item);
            }
          }
          if (changes) {
            for (const item of changes) {
              fs.purge?.(item);
            }
          }
        }
        const { fileTimeInfoEntries, contextTimeInfoEntries } = fetchTimeInfo();
        return {
          changes,
          removals,
          fileTimeInfoEntries,
          contextTimeInfoEntries,
        };
      },
    };
  }

  on(
    event: 'change',
    listener: (filename: string, mtime: number) => void,
  ): this;
  on(event: 'remove', listener: (filename: string) => void): this;
  on(
    event: 'aggregated',
    listener: (changes: Set<string>, removals: Set<string>) => void,
  ): this;
  on(
    event: 'change' | 'remove' | 'aggregated',
    listener:
      | ((filename: string, mtime: number) => void)
      | ((filename: string) => void)
      | ((changes: Set<string>, removals: Set<string>) => void),
  ): this {
    this.#events.on(event, listener as (...args: unknown[]) => void);
    return this;
  }

  once(
    event: 'change',
    listener: (filename: string, mtime: number) => void,
  ): this;
  once(event: 'remove', listener: (filename: string) => void): this;
  once(
    event: 'aggregated',
    listener: (changes: Set<string>, removals: Set<string>) => void,
  ): this;
  once(
    event: 'change' | 'remove' | 'aggregated',
    listener:
      | ((filename: string, mtime: number) => void)
      | ((filename: string) => void)
      | ((changes: Set<string>, removals: Set<string>) => void),
  ): this {
    this.#events.once(event, listener as (...args: unknown[]) => void);
    return this;
  }

  emit(event: 'change', filename: string, mtime: number): boolean;
  emit(event: 'remove', filename: string): boolean;
  emit(
    event: 'aggregated',
    changes: Set<string>,
    removals: Set<string>,
  ): boolean;
  emit(
    event: 'change' | 'remove' | 'aggregated',
    arg1: string | Set<string>,
    arg2?: number | Set<string>,
  ): boolean {
    if (event === 'aggregated') {
      // `aggregated` is a summary event, not a primitive filesystem event:
      // notify standard `on`/`once` listeners without re-dispatching it through
      // watchpack (which would trigger a rebuild), keeping it consistent with
      // the native side, where no aggregated-injection primitive exists.
      return this.#events.emit(
        'aggregated',
        arg1 as Set<string>,
        arg2 as Set<string>,
      );
    }
    if (!this.watcher) {
      return false;
    }
    const filename = arg1 as string;
    // `_onChange`/`_onRemove` emit the public `change`/`remove` events and feed
    // the aggregated change/removal sets that drive the next rebuild, matching
    // how watchpack reports a real filesystem event.
    if (event === 'change') {
      this.watcher._onChange(
        filename,
        (arg2 as number) ?? Date.now(),
        filename,
        'change',
      );
    } else {
      this.watcher._onRemove(filename, filename, 'rename');
    }
    return true;
  }
}
