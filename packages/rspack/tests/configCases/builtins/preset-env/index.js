const fs = require("fs");

it("builtins preset env", () => {
	const test = () => 1;
	expect(test()).toBe(1);
});

it("builtins preset env code check", () => {
	const content = fs.readFileSync(__filename, "utf-8");
	expect(content).not.toMatch(/\=\>/);
});
