globalThis.__order.push('ext');
export let value = 1;
export function update() {
  value = 2;
}
export const shared = 'ext';
export default 'not re-exported';
