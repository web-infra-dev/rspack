require("./a.css");
require("./b.css");
const fs = require("fs");
const path = require("path");

it("at-import-in-the-top", async () => {
	const css = await fs.promises.readFile(
		path.resolve(__dirname, "bundle0.css"),
		"utf-8"
	);

	expect(css).toMatchSnapshot();
});
