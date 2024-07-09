const fs = require("fs");
const path = require("path");
const swc = require("@swc/core");

const runtime = fs.readFileSync(
	path.resolve(__dirname, "../src/container/default.runtime.js"),
	"utf-8"
);
const downgradedRuntime = swc.transformSync(runtime, {
	jsc: {
		target: "es2015"
	}
});
const minimizedRuntime = swc.minifySync(downgradedRuntime, {
	compress: false,
	mangle: false,
	ecma: 2015
});

fs.writeFileSync(
	path.resolve(__dirname, "../dist/container/default.runtime.js"),
	minimizedRuntime.code
);
