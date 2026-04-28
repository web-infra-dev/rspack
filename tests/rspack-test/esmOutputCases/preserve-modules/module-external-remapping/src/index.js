import { EventEmitter as EventEmitterWrapped, once as onceWrapped } from './wrapped';

it('should keep real external properties when mangleExports shortens module external exports', async () => {
  const events = await import(/* webpackIgnore: true */ 'node:events');

  expect(EventEmitterWrapped).toBe(events.EventEmitter);
  expect(onceWrapped).toBe(events.once);

  const wrapped = await import(/* webpackIgnore: true */ './wrapped.mjs');

  expect(wrapped.EventEmitter).toBe(events.EventEmitter);
  expect(wrapped.once).toBe(events.once);
});
