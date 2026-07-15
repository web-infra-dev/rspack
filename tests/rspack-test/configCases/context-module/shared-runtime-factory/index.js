const syncContext = require.context("./modules", false, /\.js$/, "sync");
const weakContext = require.context("./modules", false, /\.js$/, "weak");
const emptyContext = require.context(
	"./modules",
	false,
	/never-match/,
	"sync"
);
const eagerContext = require.context("./modules", false, /\.js$/, "eager");
const asyncWeakContext = require.context(
	"./modules",
	false,
	/\.js$/,
	"async-weak"
);
const lazyOnceContext = require.context(
	"./modules",
	false,
	/\.js$/,
	"lazy-once"
);

it("keeps sync context behavior", () => {
	expect(syncContext("./a.js").value).toBe("a");
	expect(syncContext.keys()).toEqual(["./a.js"]);
	expect(syncContext.resolve("./a.js")).toBeDefined();
});

it("keeps weak context behavior", () => {
	expect(weakContext("./a.js").value).toBe("a");
	expect(weakContext.keys()).toEqual(["./a.js"]);
});

it("keeps empty sync context behavior", () => {
	expect(emptyContext.keys()).toEqual([]);
	expect(emptyContext.resolve).toBe(emptyContext);
	expect(() => emptyContext("./missing.js")).toThrow(
		"Cannot find module './missing.js'"
	);
});

it("keeps eager context behavior", async () => {
	expect((await eagerContext("./a.js")).value).toBe("a");
	await expect(eagerContext("./missing.js")).rejects.toMatchObject({
		code: "MODULE_NOT_FOUND"
	});
});

it("keeps async weak context behavior", async () => {
	expect((await asyncWeakContext("./a.js")).value).toBe("a");
});

it("keeps lazy-once context behavior", async () => {
	expect((await lazyOnceContext("./a.js")).value).toBe("a");
	expect(typeof lazyOnceContext.resolve("./a.js").then).toBe("function");
});
