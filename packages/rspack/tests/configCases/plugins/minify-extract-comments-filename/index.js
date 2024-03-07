const fs = require("fs");
const path = require("path");

import(/* webpackChunkName: "chunk" */"./chunk");

it("should minify and extract comments", () => {
	const content = fs.readFileSync(
		path.resolve(__dirname, "chunk.bundle0.js.chunk.bundle0.js.COMMENTS.txt"),
		"utf-8"
	);
	expect(content).toBe("/**\n * @preserve Some comment\n */");
});
