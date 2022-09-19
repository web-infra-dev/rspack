const fs = require("fs");

function test() {
	return 123;
}

it("basic", () => {
	expect(test()).toBe(123);
});

it("format", () => {
	const content = fs.readFileSync(__filename, "utf-8");
	expect(content).not.toMatch("\n");
});
