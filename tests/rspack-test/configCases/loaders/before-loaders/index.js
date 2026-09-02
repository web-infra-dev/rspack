it("should run a loader a beforeLoaders tap added", function () {
	expect(require("./add")).toBe("add+added");
});

it("should run a configured loader with the options a beforeLoaders tap replaced", function () {
	expect(require("./mutate")).toBe("mutate+mutated");
});

it("should not run a configured loader a beforeLoaders tap removed", function () {
	expect(require("./remove")).toBe("remove");
});

it("should keep loaders no beforeLoaders tap touched", function () {
	expect(require("./untouched")).toBe("untouched+config");
});
