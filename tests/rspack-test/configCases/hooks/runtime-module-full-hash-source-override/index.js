const fs = require("node:fs");
const path = require("node:path");

it("should preserve overridden full hash runtime module source", () => {
	expect(typeof __webpack_hash__).toBe("string");
	expect(__webpack_require__.hookedFullHash).toBe("override");

	const runtimeSource = fs.readFileSync(
		path.join(__dirname, "runtime.js"),
		"utf-8"
	);
	expect(runtimeSource).toContain('__webpack_require__.hookedFullHash = "override";');
});
