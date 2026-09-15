const fs = require("node:fs");

module.exports = function (source, sourceMap, additionalData) {
  fs.appendFileSync(this.getOptions().counterFile, "1");
  this.callback(null, source, sourceMap, additionalData);
};
