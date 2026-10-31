globalThis.chunkGroupImporterValue = module.exports = 1;
module.hot.accept();
---
globalThis.chunkGroupImporterValue = module.exports = require("../dep");
module.hot.accept();
