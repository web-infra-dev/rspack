const fs = require("fs");

function hasValueObject(obj) {
	for (const key in obj) {
		const value = obj[key];
		if (value instanceof Object) {
			return true;
		}
	}
	return false;
}

it("basic", () => {
	const obj = {};
	obj.test = {};
	obj.const = 123;
	expect(hasValueObject(obj)).toBe(true);
});

it("format", () => {
	const content = fs.readFileSync(__filename, "utf-8");
	expect(content).toMatch(/obj\.const/);
	expect(content).toMatch(/value\ instanceof\ Object/);
});
