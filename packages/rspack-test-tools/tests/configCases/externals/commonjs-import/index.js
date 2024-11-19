const fs = require("fs");
const path = require("path");
const readCase = (name)=> fs.readFileSync(path.resolve(__dirname, `${name}.js`), "utf-8");
const caseContent = readCase("case");

it("dynamic import should be preserved, others should be in commonjs external", function () {
	expect(caseContent).toContain(`import("external2-alias")`)
	expect(caseContent).toContain(`require("external1-alias")`)
	expect(caseContent).not.toContain(`require("external2-alias")`)
	expect(caseContent).toContain(`const e2 = Promise.resolve(/* import() */).then(__webpack_require__.bind(__webpack_require__, `)
});
