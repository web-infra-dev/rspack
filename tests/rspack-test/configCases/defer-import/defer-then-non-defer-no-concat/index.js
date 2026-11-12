it("execution order should be correct without module concatenation", () => {
	return import("../defer-then-non-defer/entry.js");
});
