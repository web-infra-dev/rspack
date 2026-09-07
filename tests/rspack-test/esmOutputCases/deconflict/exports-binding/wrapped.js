export const exports = 44;

// Accessing module prevents this module from being scope hoisted.
console.log.bind(module);
