export * from './ext.mjs';
export * from './other.mjs';
export let own = 1;
export function bump() {
  own = 2;
}
import './side.mjs';
