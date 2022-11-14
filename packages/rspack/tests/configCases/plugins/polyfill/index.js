it("node-polyfill", () => {
	const vm = require("vm");
	const fs = require("fs");
	const content = fs.readFileSync(__filename, "utf-8");
	expect(content).toMatch(/vm-browserify/);
	console.log("vm:", vm);
});
