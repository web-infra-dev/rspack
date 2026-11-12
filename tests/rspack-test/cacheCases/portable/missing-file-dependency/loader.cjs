const path = require("node:path");
module.exports = function (source) {
  this.addDependency(path.join(this.rootContext, "optional.config.js"));
  return source;
};
