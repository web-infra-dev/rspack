import type { Compilation } from './Compilation';
import type { Compiler } from './Compiler';
import type { Watching } from './Watching';

export type WatchCause =
  | { readonly kind: 'initial' }
  | { readonly kind: 'lazy'; readonly keys: readonly string[] }
  | {
      readonly kind: 'source';
      readonly changed: readonly string[];
      readonly removed: readonly string[];
    }
  | { readonly kind: 'unknown' };

type InvalidationCause = Exclude<WatchCause, { kind: 'initial' }>;

export interface WatchInvalidation {
  readonly watchId: string;
  readonly revision: number;
  readonly cause: WatchCause;
}

export interface RebuildOrigin {
  readonly watchId: string;
  readonly throughRevision: number;
  readonly causes: readonly WatchInvalidation[];
  readonly consumedLazyKeys: readonly string[];
}

let nextWatchId = 0n;

function copyCause(cause: WatchCause): WatchCause {
  switch (cause.kind) {
    case 'lazy':
      return Object.freeze({
        kind: 'lazy',
        keys: Object.freeze([...cause.keys]),
      });
    case 'source':
      return Object.freeze({
        kind: 'source',
        changed: Object.freeze([...cause.changed]),
        removed: Object.freeze([...cause.removed]),
      });
    case 'initial':
      return Object.freeze({ kind: 'initial' });
    case 'unknown':
      return Object.freeze({ kind: 'unknown' });
  }
}

export class WatchOriginLedger<Compilation extends object = object> {
  readonly #watchId = `watch-${++nextWatchId}`;
  readonly #notify: (invalidation: WatchInvalidation) => void;
  readonly #receipts = new WeakMap<Compilation, RebuildOrigin>();
  #revision = 0;
  #pending: WatchInvalidation[] | undefined;

  constructor(notify: (invalidation: WatchInvalidation) => void) {
    this.#notify = notify;
    this.#pending = [
      Object.freeze({
        watchId: this.#watchId,
        revision: 0,
        cause: copyCause({ kind: 'initial' }),
      }),
    ];
  }

  get revision(): number {
    return this.#revision;
  }

  get hasPendingInvalidations(): boolean {
    return this.#openPending().length > 0;
  }

  get watchId(): string {
    return this.#watchId;
  }

  invalidate(cause: InvalidationCause): WatchInvalidation {
    const pending = this.#openPending();
    if (this.#revision === Number.MAX_SAFE_INTEGER) {
      throw new RangeError('Watch invalidation revision exhausted');
    }
    const copiedCause = copyCause(cause);
    const invalidation = Object.freeze({
      watchId: this.#watchId,
      revision: ++this.#revision,
      cause: copiedCause,
    });
    pending.push(invalidation);
    this.#notify(invalidation);
    return invalidation;
  }

  begin(
    compilation: Compilation,
    throughRevision = this.#revision,
  ): RebuildOrigin | undefined {
    const pending = this.#openPending();
    const existing = this.#receipts.get(compilation);
    if (existing) return existing;
    const successorIndex = pending.findIndex(
      (event) => event.revision > throughRevision,
    );
    const causes =
      successorIndex < 0 ? pending : pending.slice(0, successorIndex);
    if (causes.length === 0) return undefined;
    const receipt: RebuildOrigin = Object.freeze({
      watchId: this.#watchId,
      throughRevision: causes[causes.length - 1]!.revision,
      causes: Object.freeze(causes),
      consumedLazyKeys: Object.freeze([]),
    });
    this.#pending = successorIndex < 0 ? [] : pending.slice(successorIndex);
    this.#receipts.set(compilation, receipt);
    return receipt;
  }

  recordConsumed(
    compilation: Compilation,
    keys: readonly string[],
  ): RebuildOrigin {
    this.#openPending();
    const previous = this.#receipts.get(compilation);
    if (!previous) {
      throw new Error('Cannot record consumption without compilation origin');
    }
    const consumedLazyKeys = Object.freeze([
      ...new Set([...previous.consumedLazyKeys, ...keys]),
    ]);
    if (consumedLazyKeys.length === previous.consumedLazyKeys.length)
      return previous;
    const receipt: RebuildOrigin = Object.freeze({
      ...previous,
      consumedLazyKeys,
    });
    this.#receipts.set(compilation, receipt);
    return receipt;
  }

  read(compilation: Compilation): RebuildOrigin | undefined {
    return this.#receipts.get(compilation);
  }

  close(): void {
    this.#pending = undefined;
  }

  #openPending(): WatchInvalidation[] {
    if (!this.#pending) throw new Error('Watch origin ledger is closed');
    return this.#pending;
  }
}

interface WatchOriginOwner {
  readonly ledger: WatchOriginLedger<Compilation>;
  readonly invalidate: (notifyChange: boolean) => void;
  nextCompilationRevision: number | undefined;
}

const lazyPreparations = new WeakMap<Compiler, () => void>();

export function registerLazyCompilation(
  compiler: Compiler,
  prepare: () => void,
): void {
  lazyPreparations.set(compiler, prepare);
}

const watches = new WeakMap<Watching, WatchOriginOwner>();
const compilations = new WeakMap<Compilation, WatchOriginLedger<Compilation>>();

export function openWatchOrigin(
  watching: Watching,
  notify: (invalidation: WatchInvalidation) => void,
  invalidate: (notifyChange: boolean) => void,
): WatchOriginLedger<Compilation> {
  const ledger = new WatchOriginLedger<Compilation>(notify);
  watches.set(watching, {
    ledger,
    invalidate,
    nextCompilationRevision: undefined,
  });
  return ledger;
}

export function closeWatchOrigin(watching: Watching): void {
  watches.get(watching)?.ledger.close();
  watches.delete(watching);
}

export function prepareWatchOrigin(watching: Watching): void {
  const owner = watches.get(watching);
  if (owner) {
    owner.nextCompilationRevision = owner.ledger.revision;
    lazyPreparations.get(watching.compiler)?.();
  }
}

export function beginWatchOrigin(
  watching: Watching | undefined,
  compilation: Compilation,
): void {
  const owner = watching && watches.get(watching);
  if (!owner || owner.nextCompilationRevision === undefined) return;
  const revision = owner.nextCompilationRevision;
  owner.nextCompilationRevision = undefined;
  if (owner.ledger.begin(compilation, revision))
    compilations.set(compilation, owner.ledger);
}

export function readWatchOrigin(
  compilation: Compilation,
): RebuildOrigin | undefined {
  return compilations.get(compilation)?.read(compilation);
}

export function consumeLazyKeys(
  watching: Watching | undefined,
  compilation: Compilation | undefined,
  keys: readonly string[],
): void {
  const ledger = watching && watches.get(watching)?.ledger;
  if (compilation && ledger?.read(compilation))
    ledger.recordConsumed(compilation, keys);
}

export function invalidateLazyCompilation(
  watching: Watching,
  keys: readonly string[],
): void {
  const owner = watches.get(watching);
  if (!owner) return;
  owner.ledger.invalidate({ kind: 'lazy', keys });
  owner.invalidate(true);
}

export function scheduleWatchRebuild(watching: Watching): void {
  const owner = watches.get(watching);
  if (!owner) return;
  if (!owner.ledger.hasPendingInvalidations)
    owner.ledger.invalidate({ kind: 'unknown' });
  owner.invalidate(false);
}

export function invalidateWatchDependency(
  watching: Watching | undefined,
): void {
  if (watching) watches.get(watching)?.ledger.invalidate({ kind: 'unknown' });
}
