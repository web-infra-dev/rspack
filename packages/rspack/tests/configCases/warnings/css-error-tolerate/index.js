const fs = require("fs");
const path = require("path");

it("build error module should have 'throw error'", () => {
	try {
		require("./index.css");
	} catch (e) {
		expect(e.message.includes("Undefined variable.")).toBe(true);
	}

	const js = fs.readFileSync(path.resolve(__dirname, "main.js"), "utf-8");
	const cssStub = /".\/index.css":.*\n\"use strict\";\n(.*)/.exec(js)[1];
	expect(cssStub.includes("throw new Error")).toBe(false);
	const css = fs.readFileSync(path.resolve(__dirname, "main.css"), "utf-8");
	expect(css.includes("throw new Error")).toBe(false);
	expect(css.includes("alpha(opacity=50);")).toBe(true);
});
