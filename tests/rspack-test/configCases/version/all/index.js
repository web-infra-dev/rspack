const path = require("path");
const fs = require("fs");

it("should inject version when use bundlerInfo.force=true", () => {
	const pattern = globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK
		? /Object\.defineProperty\(__rspack_context, "rv"/m
		: /(^|[^"'`])__webpack_require__\.rv =/m;
	expect(
		fs.readFileSync(path.join(__dirname, "bundle0.js"), "utf-8")
	).toMatch(pattern);
});
