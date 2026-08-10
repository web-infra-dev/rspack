import shared from "./shared";

const required = require("./shared");

it("should relocate ESM and CommonJS references to the same initializer", () => {
	expect(shared).toBe(required);
	expect(required.value).toBe(42);
});
