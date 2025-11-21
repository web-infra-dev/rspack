const fs = require("fs");
const path = require("path");

it("entrypoints should works", () => {
	const content = fs.readFileSync(
		path.resolve(__dirname, "inspect.txt"),
		"utf-8"
	);
	expect(content).toBe(`entry name: main
  file: bundle0.js
`);
});
