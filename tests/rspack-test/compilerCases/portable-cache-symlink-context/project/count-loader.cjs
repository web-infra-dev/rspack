const fs = require("node:fs");

module.exports = function (source) {
  fs.appendFileSync(this.getOptions().log, `${this.resourcePath}\n`);
  return source;
};
