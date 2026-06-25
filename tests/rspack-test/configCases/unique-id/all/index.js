const path = require("path");
const fs = require("fs");

it("should inject unique id when use bundlerInfo.force=true", () => {
	const source = fs.readFileSync(path.join(__dirname, "bundle0.js"), "utf-8");
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		expect(source).toMatch(/__rspack_context\.ruid =/m);
	} else {
		expect(source).toMatch(/(^|[^"'`])__webpack_require__\.ruid =/m);
	}
});
