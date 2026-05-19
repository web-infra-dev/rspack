// Use variables to force context modules
var aName = "a";
var libName = "index";

it("should respect strictThisContextOnImports for member call via context module", async () => {
	let m = await import(`./dir4/${aName}?1`);
	expect(m.f()).toBe(STRICT_THIS_CONTEXT_ON_IMPORTS ? 1 : undefined);
	expect(m.usedExports).toEqual(STRICT_THIS_CONTEXT_ON_IMPORTS ? true : ["f", "usedExports"]);
	let m2 = await import(`./dir4/lib/${libName}?1`);
	expect(m2.b.f()).toBe(1);
	expect(m2.b.usedExports).toEqual(STRICT_THIS_CONTEXT_ON_IMPORTS ? true : ["f", "usedExports"]);
	expect(m2.usedExports).toEqual(STRICT_THIS_CONTEXT_ON_IMPORTS ? true : ["b", "f", "usedExports"]);
})

it("should respect strictThisContextOnImports for member call in then via context module", async () => {
	await import(`./dir4/${aName}?2`).then(m => {
		expect(m.f()).toBe(STRICT_THIS_CONTEXT_ON_IMPORTS ? 1 : undefined);
		expect(m.usedExports).toEqual(STRICT_THIS_CONTEXT_ON_IMPORTS ? true : ["f", "usedExports"]);
	});
	await import(`./dir4/lib/${libName}?2`).then(m2 => {
		expect(m2.b.f()).toBe(1);
		expect(m2.b.usedExports).toEqual(STRICT_THIS_CONTEXT_ON_IMPORTS ? true : ["f", "usedExports"]);
		expect(m2.usedExports).toEqual(STRICT_THIS_CONTEXT_ON_IMPORTS ? true : ["b", "f", "usedExports"]);
	});
})

it("should always correctly handle this for exportsType is DefaultWithNamed and DefaultOnly via context module", async () => {
	var cjsName = "cjs";
	const cjs = await import(`./modules/${cjsName}`);
	expect(cjs.that().value).toBe(STRICT_THIS_CONTEXT_ON_IMPORTS ? 42 : undefined);
	var dataName = "data.json";
	const json = await import(`./modules/${dataName}`);
	expect(json.default.map(d => d * 2)).toEqual([2, 4, 6]);
})
