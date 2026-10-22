export default () => Promise.resolve("initial");
---
export default () => import("./consumer").then(({ namespace }) => namespace);
