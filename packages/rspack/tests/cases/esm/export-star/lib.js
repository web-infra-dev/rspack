export * from "./variables";

function _export_star(a) {
	return a + 1;
}

const variable = _export_star(1);

it("should can define variable same as helper", function () {
	expect(variable).toBe(2);
	expect(_export_star(2)).toBe(3);
});
