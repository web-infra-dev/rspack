import '../side.mjs';
if (!globalThis.order.includes('tail')) {
  throw new Error('tail must run before b');
}
