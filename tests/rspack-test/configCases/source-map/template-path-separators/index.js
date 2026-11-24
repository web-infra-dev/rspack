const values = require("./module.js");
const custom = require("custom://host/source\\name.js?option=\\value");

it("keeps the mapped module executable", () => {
	expect(values.value0).toBe(0);
	expect(values.value6).toBe(6);
	expect(custom).toBe(42);
});
