module.exports = require("./object.json?augmented");
module.exports.local = 7;
module.exports.localFunction = () => 42;
(globalThis.__prototypeMethodsUsedExports ??= {}).augmented = __webpack_exports_info__.usedExports;
