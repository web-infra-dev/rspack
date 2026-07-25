const first = require("./module-a");
const second = require("./module-b");

it("should share canonical CommonJS ESM markers", () => {
	expect(first.__esModule).toBe(true);
	expect(second.__esModule).toBe(true);
	expect(Object.keys(first)).toEqual(["value"]);
	expect(Object.getOwnPropertyDescriptor(second, "__esModule")).toMatchObject({
		configurable: false,
		enumerable: false,
		value: true,
		writable: false
	});
});
