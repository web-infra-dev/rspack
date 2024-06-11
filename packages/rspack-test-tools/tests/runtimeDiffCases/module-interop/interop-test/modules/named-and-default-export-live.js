exports.named = "named-outdated";
exports.default = "default-outdated";

Promise.resolve().then(() => {
  exports.named = "named";
  exports.default = "default";
});
