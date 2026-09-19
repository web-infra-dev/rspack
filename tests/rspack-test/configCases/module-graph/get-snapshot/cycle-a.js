import { helper } from './cycle-b';

export const cycleValue = 1;

// Read across the cycle lazily. Calling `helper()` during evaluation would
// touch `cycleValue` before its initializer has run.
export function useHelper() {
  return helper();
}
