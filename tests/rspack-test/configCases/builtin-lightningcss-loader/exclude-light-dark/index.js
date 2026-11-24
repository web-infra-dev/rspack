import './index.css'

const fs = require("node:fs");
const path = require("node:path");

it("should keep light-dark() when the lightDark feature is excluded", () => {
	const css = fs.readFileSync(
		path.resolve(__dirname, "./bundle0.css"),
		"utf-8"
	);

	expect(css).toContain('light-dark(');
	expect(css).not.toContain('--lightningcss-light');
});
