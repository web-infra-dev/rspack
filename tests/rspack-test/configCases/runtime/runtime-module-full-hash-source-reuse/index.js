const fs = require("node:fs");
const path = require("node:path");

it("should render the final full hash source for custom runtime modules", () => {
	expect(__webpack_require__.fullHashSourceReuse).toBe(__webpack_hash__);

	const runtimeSource = fs.readFileSync(
		path.join(__dirname, "runtime.js"),
		"utf-8"
	);
	expect(runtimeSource).toContain(
		`__webpack_require__.fullHashSourceReuse = ${JSON.stringify(__webpack_hash__)};`
	);
});
