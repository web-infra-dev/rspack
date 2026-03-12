// This module is outside the preserveModules root (src/)
// It stays in the original entry chunk, which should NOT
// retain the entry name after the entry module is moved out.
console.log.bind(console)
