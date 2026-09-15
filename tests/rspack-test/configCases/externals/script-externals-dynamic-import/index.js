const fs = require("fs");
const path = require("path");

const readCase = (name)=> fs.readFileSync(path.resolve(__dirname, `${name}.js`), "utf-8");

const caseContent = readCase("case");

it("dynamic import script externals module should be returned", function () {
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		expect(caseContent).toContain(`return __rspack_context.t(m, 22)`)
	} else {
		expect(caseContent).toContain(`return __webpack_require__.t(m, 22)`)
	}

});
