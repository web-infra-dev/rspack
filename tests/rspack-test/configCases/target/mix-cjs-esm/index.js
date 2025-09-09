const cjs = require("./cjs");
const esm = require("./esm");

it("should mix cjs and esm works", async () => {
	expect(await cjs.getFilePath()).toBe("cjs.js|lib/cjs.js|lib/esm.js");
	expect(await esm.getFilePath()).toBe("esm.js|lib/cjs.js|lib/esm.js");
});
