import assert from 'node:assert/strict';
import { isMainThread } from 'node:worker_threads';

export default function (context, module, options) {
  assert.deepEqual(context[Symbol.for('css-extract-rspack-plugin')], options);
  assert.equal(module, context._module);
  context.hookWorker = !isMainThread;
  context.hookOrder = [...(context.hookOrder ?? []), 'worker'];
}
