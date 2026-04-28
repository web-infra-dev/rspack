const fs = require("node:fs");
const path = require("node:path");

it("should render the final built-in get full hash runtime source", () => {
	expect(__webpack_hash__).toBeTypeOf("string");
	expect(__webpack_hash__.length > 0).toBe(true);

	const runtimeSource = fs.readFileSync(
		path.join(__dirname, "runtime.js"),
		"utf-8"
	);
	expect(runtimeSource).toContain(__webpack_hash__);
});
