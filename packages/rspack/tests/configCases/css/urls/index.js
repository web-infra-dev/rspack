const fs = require("fs");
const path = require("path");

import("./urls.css");

it("css urls should works", async () => {
	const css = await fs.promises.readFile(
		path.resolve(__dirname, "urls_css.css"),
		"utf-8"
	);
	expect(css).toMatchSnapshot();
});
