const fs = require("fs");
const path = require("path");
module.exports = function (outputDirectory, _stats) {
	const mainContent = fs.readFileSync(path.resolve(outputDirectory, "main.js"), "utf8");
	expect(mainContent).toContain(`if (oldTag) {
  oldTag.parentNode.insertBefore(linkTag, oldTag.nextSibling);
} else {
  document.head.appendChild(linkTag);
}`);
};
