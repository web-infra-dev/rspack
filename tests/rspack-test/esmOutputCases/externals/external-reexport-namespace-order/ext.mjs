if (!globalThis.ready) throw new Error('first must run before ext');
globalThis.order.push('ext');
export let value = 42;
export function update() {
  value++;
}
