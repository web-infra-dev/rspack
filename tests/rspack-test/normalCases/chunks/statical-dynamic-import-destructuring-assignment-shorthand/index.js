it("should load only used exports for shorthand destructuring assignment", async () => {
	let usedExports;
	({ usedExports } = await import("../statical-dynamic-import/dir1/a"));
	expect(usedExports).toEqual(["usedExports"]);
});

it("should load only used exports for shorthand destructuring assignment with default value", async () => {
	let a, usedExports;
	({ a = 2, usedExports } = await import("../statical-dynamic-import/dir1/a?2"));
	expect(a).toBe(1);
	expect(usedExports).toEqual(["a", "usedExports"]);
});

it("should load only used exports for mixed shorthand and renamed destructuring assignment", async () => {
	let def, usedExports;
	({ default: def, usedExports } = await import("../statical-dynamic-import/dir1/a?3"));
	expect(def).toBe(3);
	expect(usedExports).toEqual(["default", "usedExports"]);
});

it("should static analyze shorthand destructuring assignment of dynamic import variable", async () => {
	const m = await import("../statical-dynamic-import/dir1/a?4");
	let usedExports;
	({ usedExports } = m);
	expect(usedExports).toEqual(["usedExports"]);
});

it("should support nested shorthand destructuring assignment", async () => {
	let aaa, usedExports;
	({ a: { aaa, usedExports } } = await import("../statical-dynamic-import-destructuring/lib"));
	expect(aaa).toBe(1);
	expect(usedExports).toEqual(["aaa", "usedExports"]);
});

it("should not tree-shake destructuring assignment with rest element", async () => {
	let usedExports, rest;
	({ usedExports, ...rest } = await import("../statical-dynamic-import/dir1/a?5"));
	expect(usedExports).toBe(true);
	expect(rest.a).toBe(1);
});
