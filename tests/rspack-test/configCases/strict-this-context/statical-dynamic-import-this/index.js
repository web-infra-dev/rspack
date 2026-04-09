it("should respect strictThisContextOnImports for member call", async () => {
	let m = await import("./dir4/a?1");
	expect(m.f()).toBe(STRICT_THIS_CONTEXT_ON_IMPORTS ? 1 : undefined);
	expect(m.usedExports).toEqual(STRICT_THIS_CONTEXT_ON_IMPORTS ? true : ["f", "usedExports"]);
	let m2 = await import("./dir4/lib?1");
	expect(m2.b.f()).toBe(1);
	expect(m2.b.usedExports).toEqual(STRICT_THIS_CONTEXT_ON_IMPORTS ? true : ["f", "usedExports"]);
	expect(m2.usedExports).toEqual(["b", "usedExports"]);
})

it("should respect strictThisContextOnImports for member call in then", async () => {
	await import("./dir4/a?2").then(m => {
		expect(m.f()).toBe(STRICT_THIS_CONTEXT_ON_IMPORTS ? 1 : undefined);
		expect(m.usedExports).toEqual(STRICT_THIS_CONTEXT_ON_IMPORTS ? true : ["f", "usedExports"]);
	});
	await import("./dir4/lib?2").then(m2 => {
		expect(m2.b.f()).toBe(1);
		expect(m2.b.usedExports).toEqual(STRICT_THIS_CONTEXT_ON_IMPORTS ? true : ["f", "usedExports"]);
		expect(m2.usedExports).toEqual(["b", "usedExports"]);
	});
})

it("should always correctly handle this for exportsType is DefaultWithNamed and DefaultOnly", async () => {
	const cjs = await import("./cjs");
	expect(cjs.that().value).toBe(STRICT_THIS_CONTEXT_ON_IMPORTS ? 42 : undefined);
	const json = await import("./data.json");
	expect(json.default.map(d => d * 2)).toEqual([2, 4, 6]);
})
