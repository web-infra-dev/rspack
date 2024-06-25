const path = require("path");
const fs = require("fs");

it('should inject version when use bundlerInfo.force=["uniqueId"]', () => {
	expect(
		fs.readFileSync(path.join(__dirname, "bundle0.js"), "utf-8")
	).toContain(`__webpack_require__.ruid`);
});
