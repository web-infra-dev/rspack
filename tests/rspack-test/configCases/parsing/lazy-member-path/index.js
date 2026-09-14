import * as namespace from "./data";

it("should resolve qualified names without requiring owned member keys", () => {
	expect(LAZY.path.deep.leaf).toBe("defined");
	expect(LAZY["path"]["deep"]["leaf"]).toBe("defined");
	expect(typeof LAZY.path.deep.leaf).toBe("string");
});

it("should preserve long imported paths, aliases and call receivers", () => {
	expect(namespace.tree.deep.path.read()).toBe(42);
	expect(((alias) => alias.tree.deep.path.read())(namespace)).toBe(42);
	expect(namespace.nil?.[(() => { throw new Error("unreachable"); })()]).toBeUndefined();
	expect(namespace.tree.deep?.path.read?.()).toBe(42);
});

it("should preserve computed property conversion and optional boundaries", () => {
	expect(namespace.keys[1e3]).toBe("number");
	expect(namespace.keys[0xan]).toBe("bigint");
	expect(namespace.keys[true]).toBe("boolean");
	expect(namespace.keys[null]).toBe("null");
	expect(namespace.keys[/key/ig]).toBe("regexp");
	expect(namespace.keys[`a.b`]).toBe("dot");
	expect(namespace.keys["\u00e9"]).toBe("unicode");
});

it("should keep member paths attached to the AST used for a Define replacement", () => {
	// Define replacements are parsed into a temporary AST during hook dispatch.
	expect(SYNTHETIC.deep.path.read()).toBe(42);
	expect(SYNTHETIC["deep"].path.value).toBe(42);
});
