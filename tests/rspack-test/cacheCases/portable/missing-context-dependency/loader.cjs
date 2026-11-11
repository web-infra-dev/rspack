const path = require("node:path");
module.exports = function (source) {
  this.addContextDependency(path.join(this.rootContext, "optional-directory"));
  return source;
};
