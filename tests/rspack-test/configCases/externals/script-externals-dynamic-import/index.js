const fs = require("fs");
const path = require("path");

const readCase = (name)=> fs.readFileSync(path.resolve(__dirname, `${name}.js`), "utf-8");

const caseContent = readCase("case");

it("dynamic import script externals module should be returned", function () {
	expect(caseContent).toContain(`return __webpack_require__.t(m, 22)`)

});
