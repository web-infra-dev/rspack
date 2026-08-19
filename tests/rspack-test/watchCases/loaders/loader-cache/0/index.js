const value = require("./value");
const moduleA = require("./module-a");
const moduleB = require("./module-b");

it("should cache each opted-in loader until its input changes", () => {
	const step = +WATCH_STEP;
	const cachedRuns = step < 2 ? 1 : step;
	expect(value).toEqual({
		value: step < 4 ? "initial" : "changed",
		leftRuns: step + 1,
		markedRuns: cachedRuns,
		rightRuns: cachedRuns,
		sourceMap: true
	});
	expect(moduleA).toBe("module-a.js");
	expect(moduleB).toBe("module-b.js");
});
