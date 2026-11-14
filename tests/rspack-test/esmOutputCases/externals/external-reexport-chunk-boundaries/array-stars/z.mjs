if (!globalThis.order.includes('y')) throw new Error('y must run before z');
globalThis.order.push('z');
export let value = 42;
export function update() {
  value++;
}
