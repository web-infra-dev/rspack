const fs = require("node:fs");
const path = require("node:path");
module.exports = function () {
  const dir = path.join(this.rootContext, "dir");
  this.addContextDependency(dir);
  let names = [];
  try {
    names = fs.readdirSync(dir).sort();
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
  }
  return `module.exports = ${JSON.stringify(names)};`;
};
