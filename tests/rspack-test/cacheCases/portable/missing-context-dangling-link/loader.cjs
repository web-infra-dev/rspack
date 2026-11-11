const fs = require("node:fs");
const path = require("node:path");
module.exports = function () {
  const dir = path.join(this.rootContext, "config", "optional-directory");
  this.addContextDependency(dir);
  let kind = "absent";
  try {
    kind = `link to ${fs.readlinkSync(dir)}`;
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
  }
  return `module.exports = ${JSON.stringify(kind)};`;
};
