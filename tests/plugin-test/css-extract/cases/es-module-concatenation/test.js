const fs = require("fs");
const path = require("path");
module.exports = function (outputDirectory, _stats) {
  const mainContent = fs.readFileSync(path.resolve(outputDirectory, "main.js"), "utf8");
  expect(mainContent).toContain(`CONCATENATED MODULE: ./a.css`);
  expect(mainContent).toContain(`CONCATENATED MODULE: ./b.css`);
  expect(mainContent).toContain(`CONCATENATED MODULE: ./c.css`);
};
