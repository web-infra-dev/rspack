const path = require("path");
const fs = require("fs");

it('should inject version when use bundlerInfo.mode=["version"]', () => {
	expect(fs.readFileSync(path.join(__dirname, "main.js"), "utf-8")).toContain(
		`__webpack_require__.rv`
	);
});
