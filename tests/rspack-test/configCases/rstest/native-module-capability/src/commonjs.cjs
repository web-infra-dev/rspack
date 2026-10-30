const { createRequire: factory } = require("node:module");
const nativeRequire = factory(__FIXTURE_PARENT__);
exports.price = () => nativeRequire("./provider.cjs").price;
