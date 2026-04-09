import { record } from './tracker';

export function impureBodyCall() {
  return record('body-call');
}
