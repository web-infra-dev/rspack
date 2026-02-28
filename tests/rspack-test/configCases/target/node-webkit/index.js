const fs = require("fs");

it("basic", () => {
	const os = require("os");
	const cpus = os.cpus();
	expect(cpus.length).toBeGreaterThan(0);
});

it("format", () => {
	const content = fs.readFileSync(__filename, "utf-8");
	expect(content).toMatch(/.cpus/);
});
