require("./first.css");
require("./imported.css");

const fs = require("fs");
const path = require("path");

it("should hoist an `@import` hidden behind a BOM", () => {
	const css = fs.readFileSync(path.resolve(__dirname, "main.css"), "utf-8");

	expect(css).not.toContain("\uFEFF");
	// A leading BOM must not hide the `@import url` prefix: `@import` is only
	// honoured by browsers while it precedes every style rule.
	expect(css.indexOf("@import")).toBeLessThan(css.indexOf(".first"));
});
