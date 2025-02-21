const fs = require("fs");
const path = require("path");
module.exports = function (outputDirectory, _stats) {
  const mainContent = fs.readFileSync(path.resolve(outputDirectory, "common.js"), "utf8");
  expect(mainContent).toContain(`"./styleC.css": (function `);
  expect(mainContent).toContain(`"./styleD.css": (function `);
};
