import type { WatchFileSystem } from './fs';

type WatchCallback = Parameters<WatchFileSystem['watch']>[5];
type WatchCallbackArgs = Parameters<WatchCallback>;
type UndelayedWatchCallback = Parameters<WatchFileSystem['watch']>[6];
interface InternalWatchCallbacks {
  callbackUndelayed: UndelayedWatchCallback;
  onSource(kind: 'change' | 'remove', filename: string): void;
  onUndelayed: UndelayedWatchCallback;
}
type OptionalTimeInfoWatchCallback = (
  error: WatchCallbackArgs[0],
  fileTimeInfoEntries: WatchCallbackArgs[1] | undefined,
  contextTimeInfoEntries: WatchCallbackArgs[2] | undefined,
  changedFiles: WatchCallbackArgs[3],
  removedFiles: WatchCallbackArgs[4],
) => void;

// Only the original callback can opt in. Wrappers, including ones that copy
// all callback properties, retain the public contract of receiving Maps.
const internalCallbacks = new WeakMap<WatchCallback, InternalWatchCallbacks>();

export function markInternalCallback(
  callback: OptionalTimeInfoWatchCallback,
  handlers: InternalWatchCallbacks,
): OptionalTimeInfoWatchCallback {
  internalCallbacks.set(callback, handlers);
  return callback;
}

export function getInternalWatchCallbacks(
  callback: WatchCallback,
  callbackUndelayed: UndelayedWatchCallback,
): InternalWatchCallbacks | undefined {
  const handlers = internalCallbacks.get(callback);
  return handlers?.callbackUndelayed === callbackUndelayed
    ? handlers
    : undefined;
}

export function isInternalCallback(
  callback: WatchCallback,
): callback is OptionalTimeInfoWatchCallback {
  return internalCallbacks.has(callback);
}
