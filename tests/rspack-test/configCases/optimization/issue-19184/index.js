// Ported from webpack/webpack#20287.
it("should render modules in context-relative identifier order", function () {
	const ids = [];
	for (var i = 1; i <= 5; i++) {
		var moduleId = require("./files/file" + i + ".js");
		ids[i - 1] = moduleId;
	}

	const expectedIds = new Set(ids);

	const moduleIds = Object.keys(__webpack_modules__).filter((id) =>
		expectedIds.has(id)
	);
	expect(moduleIds).toEqual(ids);
});

it("should render modules in the same order across path separators", function () {
	const siblingId = require("./files/a0.js");
	const nestedId = require("./files/a/index.js");
	const expectedIds = new Set([nestedId, siblingId]);

	const moduleIds = Object.keys(__webpack_modules__).filter((id) =>
		expectedIds.has(id)
	);

	// Normalized paths sort a/index.js before a0.js because '/' < '0'.
	// Raw Windows paths reverse that order because '0' < '\\'.
	expect(moduleIds).toEqual([nestedId, siblingId]);
});
