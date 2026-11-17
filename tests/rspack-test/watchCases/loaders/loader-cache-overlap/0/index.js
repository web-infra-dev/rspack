const cases = [
	require("./input?file-false"),
	require("./input?file-true"),
	require("./input?build-false"),
	require("./input?build-true"),
	require("./input?native"),
	require("./input?mixed")
];
const removed = require("./input?removed");

it("should validate overlapping dependencies from every loader in a cached chain", () => {
	const step = +WATCH_STEP;
	const runs = [1, 2, 2, 3][step];
	const value = ["red", "blue", "blue", "green"][step];
	for (const result of cases) {
		expect(result).toEqual({ value, runs, leftRuns: runs });
	}
	// A removal in one loader disables the chain cache even if a later loader
	// registers the same dependencies again and the final set is unchanged.
	expect(removed).toEqual({ value, runs: step + 1, leftRuns: step + 1 });
});
