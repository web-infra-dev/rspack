import { record } from './tracker';

export function impureDefaultParam(value = record('default-param')) {
  return value;
}
