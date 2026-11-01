globalThis.completeHmrWithoutDeltaValue = require("./dep");
module.hot.accept();
---
globalThis.completeHmrWithoutDeltaMigrated = true;
globalThis.completeHmrWithoutDeltaValue = require("./dep");
module.hot.accept();
---
globalThis.completeHmrWithoutDeltaValue = 9;
module.hot.accept();
---
globalThis.completeHmrWithoutDeltaValue = 9;
module.hot.accept();
---
globalThis.completeHmrWithoutDeltaValue = require("./dep") + 1;
module.hot.accept();
