const load = () => require("./throwing.cjs");

it("caches and rethrows a wrapped CommonJS error", () => {
	let first;
	let second;
	try {
		load();
	} catch (error) {
		first = error;
	}
	try {
		load();
	} catch (error) {
		second = error;
	}

	expect(first).toBeInstanceOf(Error);
	expect(second).toBe(first);
	expect(globalThis.__strictCjsExecutions).toBe(1);
});
