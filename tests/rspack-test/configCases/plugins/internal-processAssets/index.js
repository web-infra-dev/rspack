const fs = require("fs");

it("plugin", () => {
	const content = fs.readFileSync(__filename, "utf-8");
	expect(content).toMatch("//banner;\n");
});
