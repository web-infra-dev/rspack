import assert from 'node:assert/strict';
const cases = [];
function test(description, build) { cases.push({ description, build }); }
import { WatchOriginLedger } from '../../../packages/rspack/src/WatchOrigin.ts';

function started() {
  const notifications = [];
  const ledger = new WatchOriginLedger(event => notifications.push(event));
  const initial = {};
  ledger.begin(initial);
  return { ledger, notifications, initial, watchId: ledger.watchId };
}

test('initial compilation has explicit initial origin at revision zero', () => {
  const notifications = [];
  const ledger = new WatchOriginLedger(event => notifications.push(event));
  const compilation = {};
  assert.deepEqual(ledger.begin(compilation), {
    watchId: ledger.watchId,
    throughRevision: 0,
    causes: [{ watchId: ledger.watchId, revision: 0, cause: { kind: 'initial' } }],
    consumedLazyKeys: [],
  });
  assert.deepEqual(notifications, []);
});

test('two lazy requests notify synchronously and coalesce without losing revisions', () => {
  const { ledger, notifications, watchId } = started();
  ledger.invalidate({ kind: 'lazy', keys: ['route/a'] });
  assert.deepEqual(notifications, [
    { watchId, revision: 1, cause: { kind: 'lazy', keys: ['route/a'] } },
  ]);
  ledger.invalidate({ kind: 'lazy', keys: ['route/b'] });
  assert.deepEqual(notifications, [
    { watchId, revision: 1, cause: { kind: 'lazy', keys: ['route/a'] } },
    { watchId, revision: 2, cause: { kind: 'lazy', keys: ['route/b'] } },
  ]);
  assert.deepEqual(ledger.begin({}), {
    watchId,
    throughRevision: 2,
    causes: [
      { watchId, revision: 1, cause: { kind: 'lazy', keys: ['route/a'] } },
      { watchId, revision: 2, cause: { kind: 'lazy', keys: ['route/b'] } },
    ],
    consumedLazyKeys: [],
  });
});

test('lazy and unknown coalescing preserves the unknown cause', () => {
  const { ledger, watchId } = started();
  ledger.invalidate({ kind: 'lazy', keys: ['a'] });
  ledger.invalidate({ kind: 'unknown' });
  assert.deepEqual(ledger.begin({}), {
    watchId,
    throughRevision: 2,
    causes: [
      { watchId, revision: 1, cause: { kind: 'lazy', keys: ['a'] } },
      { watchId, revision: 2, cause: { kind: 'unknown' } },
    ],
    consumedLazyKeys: [],
  });
});

test('source invalidation during a build belongs only to its successor', () => {
  const { ledger, watchId } = started();
  ledger.invalidate({ kind: 'lazy', keys: ['a'] });
  const running = {};
  ledger.begin(running);
  ledger.invalidate({ kind: 'source', changed: ['entry.ts'], removed: ['old.ts'] });
  assert.deepEqual(ledger.read(running), {
    watchId,
    throughRevision: 1,
    causes: [{ watchId, revision: 1, cause: { kind: 'lazy', keys: ['a'] } }],
    consumedLazyKeys: [],
  });
  assert.deepEqual(ledger.begin({}), {
    watchId,
    throughRevision: 2,
    causes: [
      { watchId, revision: 2, cause: { kind: 'source', changed: ['entry.ts'], removed: ['old.ts'] } },
    ],
    consumedLazyKeys: [],
  });
});

test('actual backend subset and stable consumed union are distinct from requested keys', () => {
  const { ledger } = started();
  ledger.invalidate({ kind: 'lazy', keys: ['a', 'b', 'a', 'c'] });
  const compilation = {};
  const before = ledger.begin(compilation);
  ledger.recordConsumed(compilation, ['b', 'b']);
  const subset = ledger.read(compilation);
  assert.deepEqual(subset?.consumedLazyKeys, ['b']);
  assert.deepEqual(before?.consumedLazyKeys, []);
  ledger.recordConsumed(compilation, ['c', 'b', 'a', 'c']);
  const union = ledger.read(compilation);
  assert.deepEqual(union?.consumedLazyKeys, ['b', 'c', 'a']);
  assert.deepEqual(subset?.consumedLazyKeys, ['b']);
  assert.equal(ledger.recordConsumed(compilation, ['a', 'b', 'c']), union);
  assert.deepEqual(union?.causes.map(event => event.cause), [
    { kind: 'lazy', keys: ['a', 'b', 'a', 'c'] },
  ]);
});

test('input arrays and all nested snapshot objects cannot mutate ledger data', () => {
  const { ledger, notifications, watchId } = started();
  const keys = ['a'];
  const changed = ['entry.ts'];
  const removed = ['old.ts'];
  ledger.invalidate({ kind: 'lazy', keys });
  ledger.invalidate({ kind: 'source', changed, removed });
  keys.push('injected');
  changed[0] = 'injected';
  removed.length = 0;
  const compilation = {};
  ledger.begin(compilation);
  const consumed = ['a'];
  ledger.recordConsumed(compilation, consumed);
  consumed.push('injected');
  const receipt = ledger.read(compilation);
  assert.equal(Reflect.set(receipt, 'throughRevision', 900), false);
  assert.equal(Reflect.set(receipt.causes, '0', {}), false);
  assert.equal(Reflect.set(receipt.consumedLazyKeys, '0', 'injected'), false);
  for (const event of receipt.causes) {
    assert.equal(Reflect.set(event, 'revision', 900), false);
    assert.equal(Reflect.set(event.cause, 'kind', 'unknown'), false);
    if (event.cause.kind === 'lazy') {
      assert.equal(Reflect.set(event.cause.keys, '0', 'injected'), false);
    }
    if (event.cause.kind === 'source') {
      assert.equal(Reflect.set(event.cause.changed, '0', 'injected'), false);
      assert.equal(Reflect.set(event.cause.removed, '0', 'injected'), false);
    }
  }
  assert.deepEqual(ledger.read(compilation), {
    watchId,
    throughRevision: 2,
    causes: [
      { watchId, revision: 1, cause: { kind: 'lazy', keys: ['a'] } },
      { watchId, revision: 2, cause: { kind: 'source', changed: ['entry.ts'], removed: ['old.ts'] } },
    ],
    consumedLazyKeys: ['a'],
  });
  assert.deepEqual(notifications.map(event => event.revision), [1, 2]);
});

test('repeat begin does not drain pending causes or overwrite consumed keys', () => {
  const { ledger, watchId } = started();
  ledger.invalidate({ kind: 'lazy', keys: ['a'] });
  const running = {};
  ledger.begin(running);
  ledger.recordConsumed(running, ['a']);
  ledger.invalidate({ kind: 'unknown' });
  assert.deepEqual(ledger.begin(running)?.consumedLazyKeys, ['a']);
  assert.deepEqual(ledger.begin({}), {
    watchId,
    throughRevision: 2,
    causes: [{ watchId, revision: 2, cause: { kind: 'unknown' } }],
    consumedLazyKeys: [],
  });
});

test('backend updates target exact compilation even after a successor begins', () => {
  const { ledger, initial } = started();
  ledger.invalidate({ kind: 'unknown' });
  const successor = {};
  ledger.begin(successor);
  ledger.recordConsumed(initial, ['initial-backend-key']);
  assert.deepEqual(ledger.read(initial)?.consumedLazyKeys, ['initial-backend-key']);
  assert.deepEqual(ledger.read(successor)?.consumedLazyKeys, []);
});

test('missing compilation and uncaused extra passes receive no invented receipt', () => {
  const { ledger } = started();
  const unknown = {};
  assert.equal(ledger.read(unknown), undefined);
  assert.equal(ledger.begin(unknown), undefined);
  assert.equal(ledger.read(unknown), undefined);
  assert.throws(() => ledger.recordConsumed(unknown, ['a']), /without compilation origin/);
});

test('closing drops pending work, freezes history, and restarting allocates new identity', () => {
  const { ledger, initial, notifications } = started();
  ledger.recordConsumed(initial, ['initial']);
  const saved = ledger.read(initial);
  ledger.invalidate({ kind: 'lazy', keys: ['discarded'] });
  ledger.close();
  ledger.close();
  assert.equal(ledger.read(initial), saved);
  assert.throws(() => ledger.invalidate({ kind: 'unknown' }), /closed/);
  assert.throws(() => ledger.begin({}), /closed/);
  assert.throws(() => ledger.recordConsumed(initial, ['late']), /closed/);
  assert.deepEqual(ledger.read(initial)?.consumedLazyKeys, ['initial']);
  assert.deepEqual(notifications.map(event => event.revision), [1]);
  const restarted = new WatchOriginLedger(() => {});
  assert.notEqual(restarted.watchId, ledger.watchId);
  assert.equal(restarted.read(initial), undefined);
  assert.deepEqual(restarted.begin({}), {
    watchId: restarted.watchId,
    throughRevision: 0,
    causes: [{ watchId: restarted.watchId, revision: 0, cause: { kind: 'initial' } }],
    consumedLazyKeys: [],
  });
  assert.equal(restarted.invalidate({ kind: 'unknown' }).revision, 1);
});

test('invalidation is queued before synchronous notification can begin compilation', () => {
  const compilation = {};
  let returned = false;
  const ledger = new WatchOriginLedger(() => { assert.equal(returned, false); ledger.begin(compilation); });
  ledger.begin({});
  ledger.invalidate({ kind: 'lazy', keys: ['a'] });
  returned = true;
  assert.deepEqual(ledger.read(compilation), {
    watchId: ledger.watchId,
    throughRevision: 1,
    causes: [{ watchId: ledger.watchId, revision: 1, cause: { kind: 'lazy', keys: ['a'] } }],
    consumedLazyKeys: [],
  });
});

test('throwing notification preserves the already observed invalidation', () => {
  const failure = new Error('observer failed');
  const ledger = new WatchOriginLedger(() => { throw failure; });
  ledger.begin({});
  assert.throws(() => ledger.invalidate({ kind: 'unknown' }), error => error === failure);
  assert.deepEqual(ledger.begin({}), {
    watchId: ledger.watchId,
    throughRevision: 1,
    causes: [{ watchId: ledger.watchId, revision: 1, cause: { kind: 'unknown' } }],
    consumedLazyKeys: [],
  });
});

test('an uncaused begin can later attach a real invalidation to the same object', () => {
  const { ledger, watchId } = started();
  const compilation = {};
  assert.equal(ledger.begin(compilation), undefined);
  ledger.invalidate({ kind: 'unknown' });
  assert.deepEqual(ledger.begin(compilation), {
    watchId, throughRevision: 1,
    causes: [{ watchId, revision: 1, cause: { kind: 'unknown' } }],
    consumedLazyKeys: [],
  });
});
export default cases;
