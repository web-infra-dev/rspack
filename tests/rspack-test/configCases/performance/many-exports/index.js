import sum from "./reexport.loader.mjs!";

it("should compile a module with many ESM exports in acceptable time", function() {
	expect(sum).toBe(499500);
});
