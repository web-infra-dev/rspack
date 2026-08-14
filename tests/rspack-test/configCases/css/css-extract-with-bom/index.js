require("./a.css");
require("./b.css");

const fs = require("fs");
const path = require("path");

it("should not leave a BOM anywhere in the extracted CSS", () => {
	const css = fs.readFileSync(path.resolve(__dirname, "raw.css"), "utf-8");
	expect(css).not.toContain("\uFEFF");
	// the rule right after the BOM must survive stripping it
	expect(css).toContain("@font-face");
});

it("should keep the rule that follows a loader-injected BOM after minifying", () => {
	const css = fs.readFileSync(path.resolve(__dirname, "main.css"), "utf-8");
	expect(css).toContain("@font-face");
});
