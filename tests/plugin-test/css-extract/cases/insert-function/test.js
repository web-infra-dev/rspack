const fs = require("fs");
const path = require("path");
module.exports = function (outputDirectory, _stats) {
  const mainContent = fs.readFileSync(path.resolve(outputDirectory, "main.js"), "utf8");
  expect(mainContent).toContain(`
				const reference = document.querySelector(".hot-reload");
				if (reference) {
					reference.parentNode.insertBefore(linkTag, reference);
				}`);
};
