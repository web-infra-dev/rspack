const fs = require("fs");
const path = require("path");

it("script type should be module", async function () {
	const content = await fs.promises.readFile(
		path.resolve(__dirname, "a.js"),
		"utf-8"
	);

	expect(content).toContain("script.timeout = 1234;");
});
