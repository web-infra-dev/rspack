export default await Promise.resolve(1);
---
export default await Promise.reject(new Error("dependency failed"));
