globalThis.completeHmrChunkValue = 1;
module.hot.accept();
---
globalThis.completeHmrChunkValue = require("./dep");
module.hot.accept();
---
globalThis.completeHmrChunkValue = 9;
module.hot.accept();
---
globalThis.completeHmrChunkValue = 9;
module.hot.accept();
---
globalThis.completeHmrChunkValue = require("./dep") + 1;
module.hot.accept();
