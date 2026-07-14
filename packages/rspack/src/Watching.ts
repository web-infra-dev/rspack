/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/Watching.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */
import type { Callback } from '@rspack/lite-tapable';

import type { Compilation, Compiler } from '.';
import { Stats } from '.';
import type { WatchInvalidationKind } from './Compilation';
import type { WatchOptions } from './config';
import type { FileSystemInfoEntry, Watcher } from './util/fs';

type PendingWatchDelta = { added: Set<string>; removed: Set<string> };

function withWatchDelta(
  dependencies: Iterable<string>,
  delta: PendingWatchDelta,
): Set<string> & PendingWatchDelta {
  return Object.assign(new Set(dependencies), delta);
}

// Merge an incremental `(added, removed)` delta into an accumulator, cancelling
// a path that is added then removed (or vice-versa) across calls.
function foldWatchDelta(
  pending: PendingWatchDelta,
  added: Iterable<string>,
  removed: Iterable<string>,
): void {
  for (const path of added) {
    if (!pending.removed.delete(path)) pending.added.add(path);
  }
  for (const path of removed) {
    if (!pending.added.delete(path)) pending.removed.add(path);
  }
}

export class Watching {
  watcher?: Watcher;
  pausedWatcher?: Watcher;
  compiler: Compiler;
  handler: Callback<Error, Stats>;
  callbacks: Callback<Error, void>[];
  watchOptions: WatchOptions;
  // @ts-expect-error  lastWatcherStartTime will be assigned with Date.now() during initialization
  lastWatcherStartTime: number;
  running: boolean;
  blocked: boolean;
  isBlocked: () => boolean;
  onChange: () => void;
  onInvalid: () => void;
  invalid: boolean;
  startTime?: number;
  #invalidReported: boolean;
  #closeCallbacks?: ((err?: Error | null) => void)[];
  #initial: boolean;
  #closed: boolean;
  #collectedChangedFiles?: Set<string>;
  #collectedRemovedFiles?: Set<string>;
  #pendingInvalidationKind?: WatchInvalidationKind;
  #pendingWatchDeps?: {
    file: PendingWatchDelta;
    context: PendingWatchDelta;
    missing: PendingWatchDelta;
  };
  suspended: boolean;

  constructor(
    compiler: Compiler,
    watchOptions: WatchOptions,
    handler: Callback<Error, Stats>,
  ) {
    this.callbacks = [];
    this.invalid = false;
    this.#invalidReported = true;
    this.blocked = false;
    this.isBlocked = () => false;
    this.onChange = () => {};
    this.onInvalid = () => {};
    this.compiler = compiler;
    this.running = false;
    this.#initial = true;
    this.#closed = false;
    this.watchOptions = watchOptions;
    this.handler = handler;
    this.suspended = false;

    // The default aggregateTimeout of watchpack is 200ms,
    // using smaller values can improve HMR performance
    if (typeof this.watchOptions.aggregateTimeout !== 'number') {
      this.watchOptions.aggregateTimeout = 5;
    }
    // Ignore watching files in node_modules to reduce memory usage and make startup faster
    if (this.watchOptions.ignored === undefined) {
      this.watchOptions.ignored = /[\\/](?:\.git|node_modules)[\\/]/;
    }

    process.nextTick(() => {
      if (this.#initial) this.#invalidate();
    });
  }

  watch(
    files: Iterable<string> & {
      added?: Iterable<string>;
      removed?: Iterable<string>;
    },
    dirs: Iterable<string> & {
      added?: Iterable<string>;
      removed?: Iterable<string>;
    },
    missing: Iterable<string> & {
      added?: Iterable<string>;
      removed?: Iterable<string>;
    },
  ) {
    this.pausedWatcher = undefined;
    // SAFETY: `watchFileSystem` is expected to be initialized.
    this.watcher = this.compiler.watchFileSystem!.watch(
      files,
      dirs,
      missing,
      this.lastWatcherStartTime,
      this.watchOptions,
      (
        err,
        fileTimeInfoEntries,
        contextTimeInfoEntries,
        changedFiles,
        removedFiles,
      ) => {
        if (err) {
          this.compiler.fileTimestamps = undefined;
          this.compiler.contextTimestamps = undefined;
          this.compiler.modifiedFiles = undefined;
          this.compiler.removedFiles = undefined;
          return this.handler(err);
        }
        if (changedFiles.size > 0 || removedFiles.size > 0) {
          this.#recordInvalidation('normal');
        }
        this.#invalidate(
          fileTimeInfoEntries,
          contextTimeInfoEntries,
          changedFiles,
          removedFiles,
        );
        this.onChange();
      },
      (fileName, changeTime) => {
        this.#recordInvalidation('normal');
        if (!this.#invalidReported) {
          this.#invalidReported = true;
          this.compiler.hooks.invalid.call(fileName, changeTime);
        }
        this.onInvalid();
      },
    );
  }

  close(callback?: () => void) {
    if (this.#closeCallbacks) {
      if (callback) {
        this.#closeCallbacks.push(callback);
      }
      return;
    }

    const finalCallback = (err: Error | null) => {
      this.running = false;
      this.#pendingInvalidationKind = undefined;
      this.compiler.__internal__watchInvalidationKind = undefined;
      this.compiler.running = false;
      this.compiler.watching = undefined;
      this.compiler.watchMode = false;
      this.compiler.modifiedFiles = undefined;
      this.compiler.removedFiles = undefined;
      this.compiler.fileTimestamps = undefined;
      this.compiler.contextTimestamps = undefined;
      // this.compiler.fsStartTime = undefined;
      const shutdown = (err: Error | null) => {
        this.compiler.hooks.watchClose.call();
        const closeCallbacks = this.#closeCallbacks!;
        this.#closeCallbacks = undefined;
        for (const cb of closeCallbacks) cb(err);
      };
      // TODO: compilation parameter support
      // if (compilation) {
      // 	const logger = compilation.getLogger("webpack.Watching");
      // 	logger.time("storeBuildDependencies");
      // 	this.compiler.cache.storeBuildDependencies(
      // 		compilation.buildDependencies,
      // 		err2 => {
      // 			logger.timeEnd("storeBuildDependencies");
      // 			shutdown(err || err2);
      // 		}
      // 	);
      // } else {
      // 	shutdown(err);
      // }
      shutdown(err);
    };

    this.#closed = true;
    if (this.watcher) {
      this.watcher.close();
      this.watcher = undefined;
    }
    if (this.pausedWatcher) {
      this.pausedWatcher.close();
      this.pausedWatcher = undefined;
    }
    this.compiler.watching = undefined;
    this.compiler.watchMode = false;
    this.#closeCallbacks = [];
    if (callback) {
      this.#closeCallbacks.push(callback);
    }
    if (this.running) {
      this.invalid = true;

      this._done = finalCallback;
    } else {
      finalCallback(null);
    }
  }

  #notifyInvalid() {
    if (!this.#invalidReported) {
      this.#invalidReported = true;
      this.compiler.hooks.invalid.call(null, Date.now());
    }
  }

  #recordInvalidation(kind: WatchInvalidationKind) {
    if (kind === 'normal' || this.#pendingInvalidationKind === undefined) {
      this.#pendingInvalidationKind = kind;
    }
  }

  invalidate(callback?: Callback<Error, void>) {
    this.__internal__invalidate('normal', callback);
  }

  /** @internal Invalidates with provenance supplied by Rspack internals. */
  __internal__invalidate(
    kind: WatchInvalidationKind,
    callback?: Callback<Error, void>,
  ) {
    if (callback) {
      this.callbacks.push(callback);
    }
    this.#recordInvalidation(kind);
    this.#notifyInvalid();
    this.onChange();
    this.#invalidate();
  }

  /** @internal Resume an invalidation already recorded by MultiCompiler. */
  __internal__resumeFromMultiCompiler() {
    this.#notifyInvalid();
    this.onChange();
    this.#invalidate();
  }

  /**
   * @internal This is not a public API yet, still unstable, might change in the future
   */
  invalidateWithChangesAndRemovals(
    changedFiles?: Set<string>,
    removedFiles?: Set<string>,
    callback?: Callback<Error, void>,
  ) {
    if (callback) {
      this.callbacks.push(callback);
    }
    this.#recordInvalidation('normal');
    this.#notifyInvalid();
    this.onChange();
    this.#invalidate(undefined, undefined, changedFiles, removedFiles);
  }

  #invalidate(
    fileTimeInfoEntries?: Map<string, FileSystemInfoEntry | 'ignore'>,
    contextTimeInfoEntries?: Map<string, FileSystemInfoEntry | 'ignore'>,
    changedFiles?: Set<string>,
    removedFiles?: Set<string>,
  ) {
    this.#mergeWithCollected(changedFiles, removedFiles);
    if (this.suspended || (this.isBlocked() && (this.blocked = true))) {
      return;
    }

    if (this.running) {
      this.invalid = true;
      return;
    }

    this.#go(
      fileTimeInfoEntries,
      contextTimeInfoEntries,
      changedFiles,
      removedFiles,
    );
  }

  #go(
    fileTimeInfoEntries?: ReadonlyMap<string, FileSystemInfoEntry | 'ignore'>,
    contextTimeInfoEntries?: ReadonlyMap<
      string,
      FileSystemInfoEntry | 'ignore'
    >,
    changedFiles?: ReadonlySet<string>,
    removedFiles?: ReadonlySet<string>,
  ) {
    this.#initial = false;
    if (this.startTime === undefined) this.startTime = Date.now();
    this.running = true;
    if (this.watcher) {
      this.pausedWatcher = this.watcher;
      this.lastWatcherStartTime = Date.now();
      this.watcher.pause();
      this.watcher = undefined;
    } else if (!this.lastWatcherStartTime) {
      this.lastWatcherStartTime = Date.now();
    }

    if (
      fileTimeInfoEntries &&
      contextTimeInfoEntries &&
      changedFiles &&
      removedFiles
    ) {
      this.#mergeWithCollected(changedFiles, removedFiles);
      this.compiler.fileTimestamps = fileTimeInfoEntries;
      this.compiler.contextTimestamps = contextTimeInfoEntries;
    } else if (this.pausedWatcher) {
      const { changes, removals, fileTimeInfoEntries, contextTimeInfoEntries } =
        this.pausedWatcher.getInfo();
      if (changes.size > 0 || removals.size > 0) {
        this.#recordInvalidation('normal');
      }
      this.#mergeWithCollected(changes, removals);
      this.compiler.fileTimestamps = fileTimeInfoEntries;
      this.compiler.contextTimestamps = contextTimeInfoEntries;
    }

    this.compiler.__internal__watchInvalidationKind =
      this.#pendingInvalidationKind;
    this.#pendingInvalidationKind = undefined;

    this.compiler.modifiedFiles = this.#collectedChangedFiles;
    this.compiler.removedFiles = this.#collectedRemovedFiles;
    this.#collectedChangedFiles = undefined;
    this.#collectedRemovedFiles = undefined;
    this.invalid = false;
    this.#invalidReported = false;
    this.compiler.hooks.watchRun.callAsync(this.compiler, (err) => {
      if (err) return this._done(err);

      const onCompiled = (
        err: Error | null,
        _compilation: Compilation | undefined,
      ) => {
        if (err) return this._done(err);

        const compilation = _compilation!;

        const needAdditionalPass = compilation.hooks.needAdditionalPass.call();
        if (needAdditionalPass) {
          compilation.needAdditionalPass = true;

          compilation.startTime = this.startTime;
          compilation.endTime = Date.now();
          const stats = new Stats(compilation);
          this.compiler.hooks.done.callAsync(stats, (err) => {
            if (err) return this._done(err, compilation);

            this.compiler.hooks.additionalPass.callAsync((err) => {
              if (err) return this._done(err, compilation);
              this.compiler.compile(onCompiled);
            });
          });
          return;
        }
        this._done(null, this.compiler._lastCompilation);
      };

      this.compiler.compile(onCompiled);
    });
  }

  // Fold a finished compilation's file/context/missing deltas into the accumulator.
  #accumulateWatchDeps(compilation: Compilation): void {
    const pending = (this.#pendingWatchDeps ??= {
      file: { added: new Set(), removed: new Set() },
      context: { added: new Set(), removed: new Set() },
      missing: { added: new Set(), removed: new Set() },
    });
    foldWatchDelta(
      pending.file,
      compilation.__internal__addedFileDependencies,
      compilation.__internal__removedFileDependencies,
    );
    foldWatchDelta(
      pending.context,
      compilation.__internal__addedContextDependencies,
      compilation.__internal__removedContextDependencies,
    );
    foldWatchDelta(
      pending.missing,
      compilation.__internal__addedMissingDependencies,
      compilation.__internal__removedMissingDependencies,
    );
  }

  /**
   * The reason why this is _done instead of #done, is that in Webpack,
   * it will rewrite this function to another function
   */
  private _done(error: Error | null, compilation?: Compilation) {
    this.running = false;

    let stats: undefined | Stats = undefined;

    const handleError = (err: Error, cbs?: Callback<Error, void>[]) => {
      this.compiler.hooks.failed.call(err);
      // this.compiler.cache.beginIdle();
      // this.compiler.idle = true;
      this.handler(err, stats);

      const callbacksToExecute = cbs || this.callbacks.splice(0);
      for (const cb of callbacksToExecute) {
        cb(err);
      }
    };

    if (error) {
      this.#pendingInvalidationKind = undefined;
      this.compiler.__internal__watchInvalidationKind = undefined;
      return handleError(error);
    }

    if (!compilation) {
      throw new Error('compilation is required if no error');
    }

    stats = new Stats(compilation);

    if (
      this.invalid &&
      !this.suspended &&
      !this.blocked &&
      !(this.isBlocked() && (this.blocked = true))
    ) {
      // Coalesced rebuild: the `watch()` delivery below is skipped, so carry
      // this build's deltas forward to the next delivered `watch()`. See #12904.
      this.#accumulateWatchDeps(compilation);
      if (compilation.watchInvalidationKind) {
        this.#recordInvalidation(compilation.watchInvalidationKind);
      }
      this.#go();
      return;
    }

    const startTime = this.startTime; // store last startTime for compilation
    // reset startTime for next compilation, before throwing error
    this.startTime = undefined;
    compilation.startTime = startTime;
    compilation.endTime = Date.now();
    const cbs = this.callbacks;
    this.callbacks = [];
    this.compiler.__internal__watchInvalidationKind = undefined;

    this.compiler.hooks.done.callAsync(stats, (err) => {
      if (err) return handleError(err, cbs);

      // Snapshot this build's watch deltas before user callbacks can invalidate.
      this.#accumulateWatchDeps(compilation);
      const pending = this.#pendingWatchDeps!;
      this.#pendingWatchDeps = undefined;

      const fileDependencies = withWatchDelta(
        compilation.fileDependencies,
        pending.file,
      );
      const contextDependencies = withWatchDelta(
        compilation.contextDependencies,
        pending.context,
      );
      const missingDependencies = withWatchDelta(
        compilation.missingDependencies,
        pending.missing,
      );

      this.handler(null, stats);

      process.nextTick(() => {
        if (!this.#closed) {
          this.watch(
            fileDependencies,
            contextDependencies,
            missingDependencies,
          );
        }
      });
      for (const cb of cbs) cb(null);
      this.compiler.hooks.afterDone.call(stats);
    });
  }

  #mergeWithCollected(
    changedFiles?: ReadonlySet<string>,
    removedFiles?: ReadonlySet<string>,
  ) {
    if (!this.#collectedChangedFiles || !this.#collectedRemovedFiles) {
      this.#collectedChangedFiles = new Set(changedFiles);
      this.#collectedRemovedFiles = new Set(removedFiles);
      return;
    }
    if (changedFiles) {
      for (const file of changedFiles) {
        this.#collectedChangedFiles.add(file);
        this.#collectedRemovedFiles.delete(file);
      }
    }
    if (removedFiles) {
      for (const file of removedFiles) {
        this.#collectedChangedFiles.delete(file);
        this.#collectedRemovedFiles.add(file);
      }
    }
  }

  suspend() {
    this.suspended = true;
  }

  resume() {
    if (this.suspended) {
      this.suspended = false;
      this.#invalidate();
    }
  }
}
