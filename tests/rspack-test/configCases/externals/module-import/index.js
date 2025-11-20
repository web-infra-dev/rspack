const fs = require("fs");
const path = require("path");

it("module-import should correctly get fallback type", function () {
	const content = fs.readFileSync(path.resolve(__dirname, "a.js"), "utf-8");
	expect(content).toContain(`import external0 from \"external0\";`); // module
	expect(content).toContain(`import * as __rspack_external_external1 from "external1"`); // module
	expect(content).toContain(`module.exports = __rspack_external_createRequire_require("external2")`); // node-commonjs
	expect(content).toContain(`import * as __rspack_external_external3 from "external3"`); // module
	expect(content).toContain(`const external3_2 = Promise.resolve(/* import() */).then`); // import
});
