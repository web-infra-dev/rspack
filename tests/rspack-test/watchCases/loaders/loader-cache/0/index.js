const value = require("./value");
const moduleA = require("./module-a");
const moduleB = require("./module-b");

it("should cache each opted-in loader until its input changes", () => {
	const step = +WATCH_STEP;
	const cachedRuns = Math.floor(step / 2) + 1;
	expect(value).toEqual({
		value: step < 2 ? "initial" : step < 4 ? "changed-2" : "changed-4",
		leftRuns: step + 1,
		markedRuns: cachedRuns,
		rightRuns: cachedRuns,
		sourceMapConsumerRuns: step + 1,
		sourceMapInput: `value-${step + 1}.js`,
		sourceMap: true
	});
	expect(moduleA).toBe("module-a.js");
	expect(moduleB).toBe("module-b.js");
});
