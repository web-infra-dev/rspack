const values = require("./module.js");

it("keeps the mapped module executable", () => {
	expect(values.value0).toBe(0);
	expect(values.value6).toBe(6);
});
