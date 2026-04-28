import { EventEmitterHoisted, onceHoisted } from './hoisted';
import { EventEmitter as EventEmitterWrapped, once as onceWrapped } from './wrapped';

it('should remap mangled module external namespace properties to real exports for hoisted and wrapped modules', async () => {
  const events = await import(/* webpackIgnore: true */ 'node:events');

  expect(EventEmitterHoisted).toBe(events.EventEmitter);
  expect(onceHoisted).toBe(events.once);

  expect(EventEmitterWrapped).toBe(events.EventEmitter);
  expect(onceWrapped).toBe(events.once);

  const hoisted = await import(/* webpackIgnore: true */ './hoisted.mjs');
  const wrapped = await import(/* webpackIgnore: true */ './wrapped.mjs');

  expect(hoisted.EventEmitterHoisted).toBe(events.EventEmitter);
  expect(hoisted.onceHoisted).toBe(events.once);

  expect(wrapped.EventEmitter).toBe(events.EventEmitter);
  expect(wrapped.once).toBe(events.once);
});
