const fs = require("fs");
const path = require("path");

const readCase = (name)=> fs.readFileSync(path.resolve(__dirname, `${name}.js`), "utf-8");

const caseContent = readCase("case");

it("dynamic import script externals module should be returned", function () {
	const isRspackRuntime = typeof __rspack_context !== "undefined";
	expect(caseContent).toContain(
		isRspackRuntime
			? `return __rspack_context.t(m, 22)`
			: `return __webpack_require__.t(m, 22)`
	)

});
