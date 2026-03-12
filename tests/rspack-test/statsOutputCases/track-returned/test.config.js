"use strict";

const fs = require("fs");
const path = require("path");

const getTestBlock = (bundle, name, nextName) => {
	const start = bundle.indexOf(`it("${name}", () => {`);
	expect(start).not.toBe(-1);

	const end = nextName
		? bundle.indexOf(`it("${nextName}", () => {`, start)
		: bundle.length;
	expect(end).not.toBe(-1);

	return bundle.slice(start, end);
};

module.exports = {
	validate() {
		const bundle = fs.readFileSync(
			path.resolve(__dirname, "../../js/statsOutput/track-returned/bundle.js"),
			"utf-8"
		);

		const labeled = getTestBlock(
			bundle,
			"should work correct for labeled statement",
			"should work correct for labeled statement break in nested switch"
		);
		expect(labeled).toContain(`__webpack_require__("./used.js?n=24")`);
		expect(labeled).toContain(`__webpack_require__("./used.js?n=25")`);
		expect(labeled).toContain(`__webpack_require__("./used.js?n=26")`);
		expect(labeled).toContain(`__webpack_require__("./used.js?n=27")`);

		const nestedSwitch = getTestBlock(
			bundle,
			"should work correct for labeled statement break in nested switch",
			"should work correct for labeled statement break in try/finally"
		);
		expect(nestedSwitch).toContain(`__webpack_require__("./used.js?n=227")`);

		const nestedTry = getTestBlock(
			bundle,
			"should work correct for labeled statement break in try/finally",
			"should still propagate labeled termination without label breaks"
		);
		expect(nestedTry).toContain(`__webpack_require__("./used.js?n=228")`);

		const noBreak = getTestBlock(
			bundle,
			"should still propagate labeled termination without label breaks",
			"should work correct for while statement"
		);
		expect(noBreak).toContain("// removed by dead control flow");
		expect(noBreak).not.toContain(`__webpack_require__("./used.js?n=229")`);
	}
};
