import binding from '@rspack/binding';
import type Watchpack from 'watchpack';
import type {
  FileSystemInfoEntry,
  InputFileSystem,
  Watcher,
  WatchFileSystem,
} from './util/fs';

/**
 * The following code is modified based on
 * https://github.com/webpack/watchpack/blob/332b55016b7c32dab4134f793ca71a5141bd10c1/lib/watchpack.js#L33-L57
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/watchpack/blob/main/LICENSE
 */
type JsWatcherIgnored = string | string[] | RegExp | undefined;

const toJsWatcherIgnored = (
  ignored: Watchpack.WatchOptions['ignored'],
): JsWatcherIgnored => {
  if (
    Array.isArray(ignored) ||
    typeof ignored === 'string' ||
    ignored instanceof RegExp
  ) {
    return ignored;
  }
  if (typeof ignored === 'function') {
    throw new Error(
      "NativeWatcher does not support using a function for the 'ignored' option",
    );
  }
  return undefined;
};

export default class NativeWatchFileSystem implements WatchFileSystem {
  #inner: binding.NativeWatcher | undefined;
  #isFirstWatch = true;
  #inputFileSystem: InputFileSystem;

  constructor(inputFileSystem: InputFileSystem) {
    this.#inputFileSystem = inputFileSystem;
  }

  watch(
    files: Iterable<string> & {
      added?: Iterable<string>;
      removed?: Iterable<string>;
    },
    directories: Iterable<string> & {
      added?: Iterable<string>;
      removed?: Iterable<string>;
    },
    missing: Iterable<string> & {
      added?: Iterable<string>;
      removed?: Iterable<string>;
    },
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
    if (
      (!files.added || typeof files.added[Symbol.iterator] !== 'function') &&
      (!files.removed || typeof files.removed[Symbol.iterator] !== 'function')
    ) {
      throw new Error("Invalid arguments: 'files'");
    }

    if (
      (!directories.added ||
        typeof directories.added[Symbol.iterator] !== 'function') &&
      (!directories.removed ||
        typeof directories.removed[Symbol.iterator] !== 'function')
    ) {
      throw new Error("Invalid arguments: 'directories'");
    }

    if (typeof callback !== 'function') {
      throw new Error("Invalid arguments: 'callback'");
    }

    if (typeof options !== 'object') {
      throw new Error("Invalid arguments: 'options'");
    }
    if (typeof callbackUndelayed !== 'function' && callbackUndelayed) {
      throw new Error("Invalid arguments: 'callbackUndelayed'");
    }

    const nativeWatcher = this.getNativeWatcher(options);

    nativeWatcher.watch(
      this.formatWatchDependencies(files),
      this.formatWatchDependencies(directories),
      this.formatWatchDependencies(missing),
      BigInt(startTime),
      (err: Error | null, result) => {
        if (err) {
          callback(err, new Map(), new Map(), new Set(), new Set());
          return;
        }
        nativeWatcher.pause();
        const changedFiles = result.changedFiles;
        const removedFiles = result.removedFiles;
        if (this.#inputFileSystem?.purge) {
          const fs = this.#inputFileSystem;
          for (const item of changedFiles) {
            fs.purge?.(item);
          }
          for (const item of removedFiles) {
            fs.purge?.(item);
          }
        }
        // TODO: add fileTimeInfoEntries and contextTimeInfoEntries
        callback(
          err,
          new Map(),
          new Map(),
          new Set(changedFiles),
          new Set(removedFiles),
        );
      },
      (fileName: string) => {
        // TODO: add real change time
        callbackUndelayed(fileName, Date.now());
      },
    );

    this.#isFirstWatch = false;

    return {
      close: () => {
        nativeWatcher.close().then(
          () => {
            // Clean up the internal reference to the native watcher to allow it to be garbage collected.
            this.#inner = undefined;
          },
          (err: unknown) => {
            console.error('Error closing native watcher:', err);
          },
        );
      },

      pause: () => {
        nativeWatcher.pause();
      },

      getInfo() {
        // This is a placeholder implementation.
        // TODO: The actual implementation should return the current state of the watcher.
        return {
          changes: new Set(),
          removals: new Set(),
          fileTimeInfoEntries: new Map(),
          contextTimeInfoEntries: new Map(),
        };
      },
    };
  }

  getNativeWatcher(options: Watchpack.WatchOptions): binding.NativeWatcher {
    if (this.#inner) {
      return this.#inner;
    }

    const nativeWatcherOptions: binding.NativeWatcherOptions = {
      followSymlinks: options.followSymlinks,
      aggregateTimeout: options.aggregateTimeout,
      pollInterval: typeof options.poll === 'boolean' ? 0 : options.poll,
      ignored: toJsWatcherIgnored(options.ignored),
    };
    const nativeWatcher = new binding.NativeWatcher(nativeWatcherOptions);
    this.#inner = nativeWatcher;

    return nativeWatcher;
  }

  triggerEvent(kind: 'change' | 'remove' | 'create', path: string) {
    this.#inner?.triggerEvent(kind, path);
  }

  formatWatchDependencies(
    dependencies: Iterable<string> & {
      added?: Iterable<string>;
      removed?: Iterable<string>;
    },
  ): [string[], string[]] {
    if (this.#isFirstWatch) {
      // if we first watch, we should pass all dependencies
      return [Array.from(dependencies), []];
    } else {
      // On subsequent watches, we only need to pass incremental changes:
      // [added dependencies, removed dependencies]
      return [
        Array.from(dependencies.added ?? []),
        Array.from(dependencies.removed ?? []),
      ];
    }
  }
}
