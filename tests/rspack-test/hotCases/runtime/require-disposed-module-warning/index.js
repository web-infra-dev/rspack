const expectWarning = require("@rspack/test-tools/helper/util/expectWarningFactory")();
const getInner = require("./module");

it("should print correct warning messages when a disposed module is required", async () => {
	await NEXT_HMR();
	getInner();
	expectWarning(
		/^\[HMR] unexpected require\(\.\/a.js\) from disposed module \.\/module\.js$/,
		/^\[HMR] unexpected require\(\.\/a.js\) to disposed module$/
	);
	const getInnerUpdated = require("./module");
	getInnerUpdated();
	expectWarning();

});

module.hot.accept("./module");
