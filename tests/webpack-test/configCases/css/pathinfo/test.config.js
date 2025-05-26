const fs = require("fs");
const path = require("path");

module.exports = {
	moduleScope(scope) {
		const link = scope.window.document.createElement("link");
		link.rel = "stylesheet";
		link.href = "bundle0.css";
		scope.window.document.head.appendChild(link);
	},
	findBundle: function (i, options) {
		const source = fs.readFileSync(
			path.resolve(options.output.path, "bundle0.css"),
			"utf-8"
		);

		// CHANGE: readable identifier does not starts with `css: `
		// because rspack use NormalModule to render css
		if (
			!source.includes(`/*!****************************!*\\
  !*** ./style-imported.css ***!
  \\****************************/`) &&
			!source.includes(`/*!*******************!*\\
  !*** ./style.css ***!
  \\*******************/`)
		) {
			throw new Error("The `pathinfo` option doesn't work.");
		}

		return ["./style2_css.bundle0.js", "./bundle0.js"];
	}
};
