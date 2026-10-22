it("should restore context dependencies and async blocks in every mode", async () => {
	const sync = require.context("./sync", true, /\.js$/, "sync");
	expect(sync.keys().sort()).toEqual(["./a.js", "./nested/b.js"]);
	expect(sync("./a.js")).toBe("a");
	expect(sync("./nested/b.js")).toBe("b");

	const eager = require.context("./sync", false, /\.js$/, "eager");
	expect(eager.keys()).toEqual(["./a.js"]);
	expect(await eager("./a.js")).toBe("a");

	const weak = require.context("./sync", false, /\.js$/, "weak");
	expect(weak.keys()).toEqual(["./a.js"]);
	expect(weak("./a.js")).toBe("a");

	const asyncWeak = require.context("./sync", false, /\.js$/, "async-weak");
	expect(asyncWeak.keys()).toEqual(["./a.js"]);
	expect(await asyncWeak("./a.js")).toBe("a");

	// Separate directories keep these modules in async chunks on every compiler.
	const lazy = require.context("./lazy", true, /\.js$/, "lazy");
	expect(lazy.keys().sort()).toEqual(["./a.js", "./b.js"]);
	expect(await lazy("./a.js")).toBe("lazy a");
	expect(await lazy("./b.js")).toBe("lazy b");

	const lazyOnce = require.context("./lazy-once", true, /\.js$/, "lazy-once");
	expect(lazyOnce.keys().sort()).toEqual(["./a.js", "./b.js"]);
	expect(await lazyOnce("./a.js")).toBe("lazy-once a");
	expect(await lazyOnce("./b.js")).toBe("lazy-once b");

	const empty = require.context("./sync", false, /missing\.js$/);
	expect(empty.keys()).toEqual([]);
	expect(() => empty("./missing.js")).toThrow("Cannot find module");
});
