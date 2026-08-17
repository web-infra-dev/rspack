const value = require("./value");

it("should cache a loader chain until the resource changes", () => {
	if (+WATCH_STEP < 3) {
		expect(value).toEqual({
			value: "initial",
			leftRuns: +WATCH_STEP + 1,
			markedRuns: 1,
			rightRuns: 1,
			sourceMap: true,
			additionalData: true
		});
	} else {
		expect(value).toEqual({
			value: "changed",
			leftRuns: 4,
			markedRuns: 2,
			rightRuns: 2,
			sourceMap: true,
			additionalData: true
		});
	}
});
