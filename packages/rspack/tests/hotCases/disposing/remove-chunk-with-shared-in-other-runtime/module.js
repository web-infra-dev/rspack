export default () => new Worker(new URL("./chunk2.js", import.meta.url)); // TODO(ahabhgk): should be "./chunk2" WorkerDependency instead of URLDependency
---
export default 42;
