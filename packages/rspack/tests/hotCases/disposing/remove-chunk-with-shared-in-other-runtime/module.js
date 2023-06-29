export default () => new Worker(new URL("./chunk2.js", import.meta.url));
---
export default 42;
