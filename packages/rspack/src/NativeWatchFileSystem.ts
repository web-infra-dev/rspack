import { EventEmitter } from 'node:events';
import binding from '@rspack/binding';
import type Watchpack from 'watchpack';
import type {
  ExistenceOnlyTimeEntry,
  FileSystemInfoEntry,
  InputFileSystem,
  TimeInfoEntries,
  Watcher,
  WatchFileSystem,
} from './util/fs';
import { isInternalCallback } from './util/watchTimeInfo';

/**
 * The following code is modified based on
 * https://github.com/webpack/watchpack/blob/332b55016b7c32dab4134f793ca71a5141bd10c1/lib/watchpack.js#L33-L57
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/watchpack/blob/main/LICENSE
 */
type JsWatcherIgnored =
  string | string[] | RegExp | ((entry: string) => boolean) | undefined;

const toJsWatcherIgnored = (
  ignored: Watchpack.WatchOptions['ignored'],
): JsWatcherIgnored => {
  if (
    Array.isArray(ignored) ||
    typeof ignored === 'string' ||
    ignored instanceof RegExp ||
    typeof ignored === 'function'
  ) {
    return ignored;
  }
  return undefined;
};

/** watchpack's existence-only time entry (`{}`): known to exist, no time info. */
const EXISTENCE_ONLY_TIME_ENTRY: ExistenceOnlyTimeEntry = Object.freeze({});

/** Rebuild watchpack's `TimeInfoEntries` from the flattened native rows. */
const toTimeInfoEntries = (
  rows: binding.NativeTimeInfoEntry[],
): TimeInfoEntries => {
  const entries: TimeInfoEntries = new Map();
  for (const row of rows) {
    if (row.existenceOnly) {
      entries.set(row.path, EXISTENCE_ONLY_TIME_ENTRY);
    } else if (row.safeTime == null) {
      entries.set(row.path, null);
    } else {
      const entry: FileSystemInfoEntry = { safeTime: row.safeTime };
      if (row.timestamp != null) {
        entry.timestamp = row.timestamp;
      }
      if (row.accuracy != null) {
        entry.accuracy = row.accuracy;
      }
      entries.set(row.path, entry);
    }
  }
  return entries;
};

/**
 * Minimal watchpack-compatible shim exposed as `NativeWatchFileSystem.watcher`.
 *
 * It proxies the watchpack-private surface that `ts-checker-rspack-plugin` and
 * `fork-ts-checker-webpack-plugin` rely on — `on`/`once` for `change`/`remove`,
 * plus `_onChange`/`_onRemove` to inject events — onto the native watcher, so
 * those plugins keep working under `experiments.nativeWatcher` unmodified.
 * The times API (`getTimes`/`getTimeInfoEntries`/`collectTimeInfoEntries`) is
 * forwarded to the owning file system, which reads it off the native watcher.
 *
 * APIs that iterate `fileWatchers`/`directoryWatchers` are intentionally not
 * supported; plugins needing those should use `WatchFileSystem.emit` instead.
 */
class NativeWatcherShim extends EventEmitter {
  #fileSystem: NativeWatchFileSystem;

  constructor(fileSystem: NativeWatchFileSystem) {
    super();
    this.#fileSystem = fileSystem;
  }

  _onChange(
    item: string,
    _mtime?: number,
    file?: string,
    _type?: string,
  ): void {
    this.#fileSystem.triggerEvent('change', file ?? item);
  }

  _onRemove(item: string, file?: string, _type?: string): void {
    this.#fileSystem.triggerEvent('remove', file ?? item);
  }

  getTimes(): Record<string, number | null> {
    return this.#fileSystem.getTimes();
  }

  getTimeInfoEntries(): TimeInfoEntries {
    return this.#fileSystem.getTimeInfoEntries();
  }

  collectTimeInfoEntries(
    fileTimestamps: TimeInfoEntries,
    directoryTimestamps: TimeInfoEntries,
  ): void {
    this.#fileSystem.collectTimeInfoEntries(
      fileTimestamps,
      directoryTimestamps,
    );
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
      fileTimeInfoEntries: TimeInfoEntries,
      contextTimeInfoEntries: TimeInfoEntries,
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
    const watcher = new NativeWatcherShim(this);
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
        if (isInternalCallback(callback)) {
          // Watching reads fresh timestamps through getInfo when the build starts.
          callback(null, undefined, undefined, changes, removals);
          return;
        }
        const { fileTimeInfoEntries, contextTimeInfoEntries } =
          this.#fetchTimeInfo(nativeWatcher);
        callback(
          err,
          fileTimeInfoEntries,
          contextTimeInfoEntries,
          changes,
          removals,
        );
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
        // Detach immediately: a closed native watcher rejects further watch()
        // calls, so a later compiler.watch() must get a fresh instance with a
        // full (non-incremental) registration.
        if (this.#inner === nativeWatcher) {
          this.#inner = undefined;
          this.#isFirstWatch = true;
        }
        nativeWatcher.close().catch((err: unknown) => {
          console.error('Error closing native watcher:', err);
        });
      },

      pause: () => {
        nativeWatcher.pause();
      },

      getInfo: () => {
        // Like watchpack after it emitted `aggregated`: what is pending here
        // is only what arrived since, i.e. while paused.
        const { changedFiles, removedFiles } = nativeWatcher.takeAggregated();
        const changes = new Set(changedFiles);
        const removals = new Set(removedFiles);
        if (this.#inputFileSystem?.purge) {
          const fs = this.#inputFileSystem;
          for (const item of removals) {
            fs.purge?.(item);
          }
          for (const item of changes) {
            fs.purge?.(item);
          }
        }
        const { fileTimeInfoEntries, contextTimeInfoEntries } =
          this.#fetchTimeInfo(nativeWatcher);
        return {
          changes,
          removals,
          fileTimeInfoEntries,
          contextTimeInfoEntries,
        };
      },
    };
  }

  #fetchTimeInfo(nativeWatcher: binding.NativeWatcher): {
    fileTimeInfoEntries: TimeInfoEntries;
    contextTimeInfoEntries: TimeInfoEntries;
  } {
    const { fileTimestamps, directoryTimestamps } =
      nativeWatcher.collectTimeInfoEntries();
    return {
      fileTimeInfoEntries: toTimeInfoEntries(fileTimestamps),
      contextTimeInfoEntries: toTimeInfoEntries(directoryTimestamps),
    };
  }

  /**
   * watchpack's times API, served from the native watcher's registered paths.
   * Empty before the first `watch()` and after `close()`.
   */
  getTimes(): Record<string, number | null> {
    const times: Record<string, number | null> = Object.create(null);
    if (!this.#inner) {
      return times;
    }
    const { fileTimestamps, directoryTimestamps } =
      this.#inner.collectTimeInfoEntries();
    for (const row of fileTimestamps) {
      // Like watchpack: a file reports the later of its safe time and
      // timestamp, an absent path null, an existence-only entry nothing.
      if (row.existenceOnly) {
        continue;
      }
      times[row.path] =
        row.safeTime == null
          ? null
          : Math.max(row.safeTime, row.timestamp ?? row.safeTime);
    }
    for (const row of directoryTimestamps) {
      times[row.path] = row.safeTime ?? null;
    }
    return times;
  }

  getTimeInfoEntries(): TimeInfoEntries {
    const entries: TimeInfoEntries = new Map();
    // watchpack fills both tables into one map here; a context entry, written
    // last, wins over a file entry for the same path.
    this.collectTimeInfoEntries(entries, entries);
    return entries;
  }

  collectTimeInfoEntries(
    fileTimestamps: TimeInfoEntries,
    directoryTimestamps: TimeInfoEntries,
  ): void {
    if (!this.#inner) {
      return;
    }
    const { fileTimeInfoEntries, contextTimeInfoEntries } = this.#fetchTimeInfo(
      this.#inner,
    );
    for (const [path, entry] of fileTimeInfoEntries) {
      fileTimestamps.set(path, entry);
    }
    for (const [path, entry] of contextTimeInfoEntries) {
      directoryTimestamps.set(path, entry);
    }
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
