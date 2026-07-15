const syncContext = require.context("./modules", false, /\.js$/, "sync");
const weakContext = require.context("./modules", false, /\.js$/, "weak");
const emptyContext = require.context(
	"./modules",
	false,
	/never-match/,
	"sync"
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
