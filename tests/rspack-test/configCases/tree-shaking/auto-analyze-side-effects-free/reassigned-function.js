import { record } from './tracker';

export function reassignedFunction() {
  return 1;
}

reassignedFunction = () => record('reassigned');
