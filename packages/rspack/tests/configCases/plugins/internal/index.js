const fs = require("fs");
console.log("fs:", fs);
it("plugin", () => {
	console.log("module:", module);
	const content = fs.readFileSync(__filename, "utf-8");
	console.log("content", content);
	expect(content).toMatch("//banner");
});
