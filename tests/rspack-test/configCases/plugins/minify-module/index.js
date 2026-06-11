const fs = require("fs");
const path = require("path");

it("should minify outputModule", async () => {
	const out = await fs.promises.readFile(
		path.join(__dirname, "./module.mjs"),
		"utf-8"
	);
	expect(
		out.startsWith('import s from"https://test.rspack.rs/test.js"')
	).toBe(true);
});
