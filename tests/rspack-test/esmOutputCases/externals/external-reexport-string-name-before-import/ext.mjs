globalThis.__order = globalThis.__order || [];
globalThis.__order.push('ext');
export const foo = 42;
export { foo as "x-y" };
