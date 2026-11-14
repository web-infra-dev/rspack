if (!globalThis.__order.includes('ext')) {
  throw new Error('ext must initialize before other');
}
globalThis.__order.push('other');
export const other = 42;
export const shared = 'other';
