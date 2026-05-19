const fs = require("fs");
const path = require("path");

it("should build ESM library with Worker without errors", () => {
	const mainChunk = fs.readFileSync(
		path.resolve(__dirname, "main.js"),
		"utf-8"
	);
	// The main chunk should contain the greet export
	expect(mainChunk).toContain("greet");
});
