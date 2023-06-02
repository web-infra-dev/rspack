const fs = require("fs");

it("builtins preset env", () => {
	const test = () => 1;
	expect(test()).toBe(1);
});

it("builtins preset env code check", () => {
	const content = fs.readFileSync(__filename, "utf-8");
	expect(content).not.toMatch(/\=\>/);
});

it("should transfrom arrow", () => {
	const obj = {};

	function fn() {
		expect(this).toBe(obj);
		expect(arguments[0]).toBe(1);
		const b = () => {
			expect(this).toBe(obj);
			expect(arguments[0]).toBe(1);
		};
		b();
	}

	fn.apply(obj, [1]);
});
