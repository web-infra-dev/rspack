const nativeModule = require("node:".concat("module"));
exports.price = () =>
  nativeModule.createRequire(__FIXTURE_PARENT__)("./provider.cjs").price;
