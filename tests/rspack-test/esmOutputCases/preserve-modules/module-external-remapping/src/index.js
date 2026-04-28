import * as eventsNamespace from 'node:events';
import { EventEmitter as EventEmitterWrapped, once as onceWrapped } from './wrapped';

it('should remap mangled module external namespace properties to real exports', async () => {
  const events = await import(/* webpackIgnore: true */ 'node:events');

  expect(eventsNamespace.EventEmitter).toBe(events.EventEmitter);
  expect(eventsNamespace.once).toBe(events.once);

  expect(EventEmitterWrapped).toBe(events.EventEmitter);
  expect(onceWrapped).toBe(events.once);

  const wrapped = await import(/* webpackIgnore: true */ './wrapped.mjs');

  expect(wrapped.EventEmitter).toBe(events.EventEmitter);
  expect(wrapped.once).toBe(events.once);
});
