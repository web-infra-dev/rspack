import { a as EventEmitterWrapped, b as onceWrapped } from './wrapped';

it('should keep real module external export names after remapping wrapped module exports', async () => {
  const events = await import(/* webpackIgnore: true */ 'node:events');

  expect(EventEmitterWrapped).toBe(events.EventEmitter);
  expect(onceWrapped).toBe(events.once);

  const wrapped = await import(/* webpackIgnore: true */ './wrapped.mjs');

  expect(wrapped.a).toBe(events.EventEmitter);
  expect(wrapped.b).toBe(events.once);
});
