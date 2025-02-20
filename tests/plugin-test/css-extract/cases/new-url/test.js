const fs = require("fs");
const path = require("path");
module.exports = function (outputDirectory, _stats) {
  const mainContent = fs.readFileSync(path.resolve(outputDirectory, "main.js"), "utf8");
  expect(mainContent).toContain(`new URL(/* asset import */__webpack_require__("./style.css"), __webpack_require__.b)`);
};
