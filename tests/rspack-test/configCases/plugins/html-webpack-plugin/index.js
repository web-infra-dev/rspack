const fs = require("fs");
const path = require("path");

it("html-webpack-plugin + html-loader", () => {
	const pugDist = fs.readFileSync(
		path.resolve(__dirname, "pug-index.html"),
		"utf-8"
	);
	expect(pugDist).toContain("Pug Template");
	const htmlDist = fs.readFileSync(
		path.resolve(__dirname, "html-index.html"),
		"utf-8"
	);
	expect(htmlDist).toContain("Html Template");
});
