export * from './ext.mjs';
import { next } from './middle.mjs';
export * from './other.mjs';
import { initial } from './last.mjs';

export let own = initial;
export function bump() {
  own = next(own);
}
