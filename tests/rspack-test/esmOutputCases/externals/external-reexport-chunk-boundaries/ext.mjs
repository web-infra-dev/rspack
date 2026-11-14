if (!globalThis.ready) throw new Error('setup or side must run before ext');
globalThis.order.push('ext');
export let value = 42;
export function update() {
  value++;
}
export { value as 'x-y' };
