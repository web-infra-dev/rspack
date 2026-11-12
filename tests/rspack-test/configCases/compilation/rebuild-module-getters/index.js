import { a, blocksError, dependenciesError } from "./a";

it("module getters should fail gracefully while the module graph is under construction", () => {
	expect(a).toEqual(1);
	expect(blocksError).toContain(
		"unavailable while the module graph is under construction",
	);
	expect(dependenciesError).toContain(
		"unavailable while the module graph is under construction",
	);
});
