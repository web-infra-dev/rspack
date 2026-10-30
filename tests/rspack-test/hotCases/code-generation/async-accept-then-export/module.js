export function then(resolve) { resolve({ default: "wrong namespace" }); }
export default await Promise.resolve(1);
---
export function then(resolve) { resolve({ default: "wrong namespace" }); }
export default await Promise.resolve(2);
