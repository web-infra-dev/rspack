if (!globalThis.ready) throw Error('setup must run first');
globalThis.__order = globalThis.__order || [];
globalThis.__order.push('ext');
export const value = 42;
