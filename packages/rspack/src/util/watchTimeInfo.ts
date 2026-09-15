import type { WatchFileSystem } from './fs';

type WatchCallback = Parameters<WatchFileSystem['watch']>[5];
type WatchCallbackArgs = Parameters<WatchCallback>;
type DeferredWatchCallback = (
  error: WatchCallbackArgs[0],
  fileTimeInfoEntries: WatchCallbackArgs[1] | undefined,
  contextTimeInfoEntries: WatchCallbackArgs[2] | undefined,
  changedFiles: WatchCallbackArgs[3],
  removedFiles: WatchCallbackArgs[4],
) => void;

// Only the original callback can opt in. Wrappers, including ones that copy
// all callback properties, retain the public contract of receiving Maps.
const deferredCallbacks = new WeakSet<WatchCallback>();

export function deferWatchTimeInfo(
  callback: DeferredWatchCallback,
): DeferredWatchCallback {
  deferredCallbacks.add(callback);
  return callback;
}

export function canDeferWatchTimeInfo(
  callback: WatchCallback,
): callback is DeferredWatchCallback {
  return deferredCallbacks.has(callback);
}
