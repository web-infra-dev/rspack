const path = require("path");
const fs = require("fs");

it("should only has one property", () => {
	const p = path.resolve(__dirname, "./css.js");
	const content = fs.readFileSync(p, "utf-8");

	// this output should align to
	// `webpack` + `css-loader+modules.exportLocationConvention: "camelCase"`
	// rather than `webpack+experiments.css`
	expect(content.match(/"xxx": ".*"/g).length).toBe(1);
});
