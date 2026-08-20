const originalFactory = __rspack_modules["./empty.js"];
__rspack_modules["./empty.js"] = (module, ...args) => {
  originalFactory(module, ...args);
  module.exports.rspackModulesValue = "rspack modules";
};
