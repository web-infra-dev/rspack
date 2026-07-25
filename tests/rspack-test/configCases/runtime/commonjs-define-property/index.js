const first = require("./module-a");
const second = require("./module-b");

it("should share CommonJS export property definitions", () => {
	expect(first.value).toBe("a");
	expect(second.value).toBe("b");
	expect(Object.getOwnPropertyDescriptor(first, "value")).toMatchObject({
		configurable: false,
		enumerable: true
	});
	expect(Object.keys(second)).toEqual([]);
});
