import "./index.css";

const fs = require("node:fs");
const path = require("node:path");

it("should preserve license comments", () => {
	const css = fs.readFileSync(
		path.resolve(__dirname, "bundle0.css"),
		"utf-8"
	);

	expect(css).toContain("/*! Copyright example */");
});
