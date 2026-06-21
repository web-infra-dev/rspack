import { EventEmitter } from 'node:events';
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

/**
 * Minimal watchpack-compatible shim exposed as `NativeWatchFileSystem.watcher`.
 *
 * It proxies the watchpack-private surface that `ts-checker-rspack-plugin` and
 * `fork-ts-checker-webpack-plugin` rely on — `on`/`once` for `change`/`remove`,
 * plus `_onChange`/`_onRemove` to inject events — onto the native watcher, so
 * those plugins keep working under `experiments.nativeWatcher` unmodified.
 *
 * APIs that iterate `fileWatchers`/`directoryWatchers` are intentionally not
 * supported; plugins needing those should use `WatchFileSystem.emit` instead.
 */
class NativeWatcherShim extends EventEmitter {
  #trigger: (kind: 'change' | 'remove', path: string) => void;

  constructor(trigger: (kind: 'change' | 'remove', path: string) => void) {
    super();
    this.#trigger = trigger;
  }

  _onChange(
    item: string,
    _mtime?: number,
    file?: string,
    _type?: string,
  ): void {
    this.#trigger('change', file ?? item);
  }

  _onRemove(item: string, file?: string, _type?: string): void {
    this.#trigger('remove', file ?? item);
  }
}

export default class NativeWatchFileSystem implements WatchFileSystem {
  #inner: binding.NativeWatcher | undefined;
  #isFirstWatch = true;
  #inputFileSystem: InputFileSystem;
  // Long-lived emitter backing the `on`/`once` API, so listeners registered
  // once keep receiving events across watch cycles.
  #events = new EventEmitter();
  // Recreated on every `watch()` call to mirror watchpack's per-cycle watcher
  // instance: consumers that re-attach to `.watcher` each cycle (ts-checker)
  // don't accumulate listeners on a stale shim.
  #watcher: NativeWatcherShim | undefined;

  constructor(inputFileSystem: InputFileSystem) {
    this.#inputFileSystem = inputFileSystem;
  }

  // Backward-compatible accessor: lets plugins that reach for the underlying
  // watchpack instance (e.g. ts-checker) find a compatible event surface.
  // `undefined` before the first `watch()`, like NodeWatchFileSystem.watcher.
  get watcher(): NativeWatcherShim | undefined {
    return this.#watcher;
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

    // Fresh shim per cycle (see field comment). Events are emitted to both the
    // long-lived `#events` (the `on`/`once` API) and this cycle's shim (the
    // `.watcher` surface).
    const watcher = new NativeWatcherShim((kind, path) =>
      this.#inner?.triggerEvent(kind, path),
    );
    this.#watcher = watcher;

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
        const changes = new Set(changedFiles);
        const removals = new Set(removedFiles);
        // Mirror watchpack's public `aggregated` event (the batched summary
        // delivered after the aggregate timeout) on both the standard
        // `on`/`once` API and the watchpack-compatible `.watcher` shim. Emitted
        // before `callback`, which synchronously starts the next rebuild, so
        // listeners observe the batch before compilation — matching the node
        // path, where the forwarded `aggregated` runs before its rebuild callback.
        this.#events.emit('aggregated', changes, removals);
        watcher.emit('aggregated', changes, removals);
        callback(err, new Map(), new Map(), changes, removals);
      },
      (event) => {
        if (event.kind === 'change') {
          // The native watcher reports paths without an mtime, so events are
          // stamped with their arrival time.
          const mtime = Date.now();
          callbackUndelayed(event.path, mtime);
          this.#events.emit('change', event.path, mtime);
          watcher.emit('change', event.path, mtime);
        } else {
          this.#events.emit('remove', event.path);
          watcher.emit('remove', event.path);
        }
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
    this.#events.on(event, listener);
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
    this.#events.once(event, listener);
    return this;
  }

  emit(event: 'change', filename: string, mtime: number): boolean;
  emit(event: 'remove', filename: string): boolean;
  emit(
    event: 'aggregated',
    changes: Set<string>,
    removals: Set<string>,
  ): boolean;
  // `mtime` is accepted for parity with the node implementation but cannot be
  // carried through the native watcher pipeline, which re-stamps the event with
  // its arrival time; the injected `change` is reported with that timestamp.
  emit(
    event: 'change' | 'remove' | 'aggregated',
    arg1: string | Set<string>,
    arg2?: number | Set<string>,
  ): boolean {
    if (event === 'aggregated') {
      const changes = arg1 as Set<string>;
      const removals = arg2 as Set<string>;
      // `aggregated` is a summary event with no native injection primitive, so
      // this re-broadcasts it to listeners (standard API + `.watcher` shim)
      // rather than driving a rebuild.
      const notified = this.#events.emit('aggregated', changes, removals);
      const shimNotified =
        this.#watcher?.emit('aggregated', changes, removals) ?? false;
      return notified || shimNotified;
    }
    if (!this.#inner) {
      return false;
    }
    // Route through the native watcher so the injected event flows back through
    // the normal pipeline (driving a rebuild and re-emitting `change`/`remove`),
    // mirroring watchpack's `_onChange`/`_onRemove`.
    this.#inner.triggerEvent(event, arg1 as string);
    return true;
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
