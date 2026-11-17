const path = require("node:path");
module.exports = function (source) {
  this.addContextDependency(path.join(this.rootContext, "config", "optional-directory"));
  return source;
};
