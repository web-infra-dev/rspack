import * as eventsNamespace from 'node:events';

export const EventEmitterHoisted = eventsNamespace.EventEmitter;
export const onceHoisted = eventsNamespace.once;
