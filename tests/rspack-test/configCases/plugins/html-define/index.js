const fs = require("fs");
const path = require("path");

it("html-webpack-plugin + define-plugin", () => {
	const html = fs.readFileSync(path.resolve(__dirname, "index.html"), "utf-8");
	expect(html).toContain("CUSTOM TITLE");
});
