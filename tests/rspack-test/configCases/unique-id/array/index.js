const path = require("path");
const fs = require("fs");

it('should inject unique id when use bundlerInfo.force=["uniqueId"]', () => {
	const pattern = globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK
		? /Object\.defineProperty\(__rspack_context, "ruid"/m
		: /(^|[^"'`])__webpack_require__\.ruid =/m;
	expect(
		fs.readFileSync(path.join(__dirname, "bundle0.js"), "utf-8")
	).toMatch(pattern);
});
