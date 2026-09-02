import * as reexports from "./reexports";

it("should preserve dynamic reexport semantics without const and arrow functions", () => {
	const names = "abcdefghijklmnop".split("");

	expect(Object.keys(reexports).sort()).toEqual([...names, "setA"].sort());
	for (const [index, name] of names.entries()) {
		expect(reexports[name]).toBe(index + 1);
	}
	expect(Object.prototype.hasOwnProperty.call(reexports, "default")).toBe(
		false
	);

	reexports.setA(101);
	expect(reexports.a).toBe(101);
});
