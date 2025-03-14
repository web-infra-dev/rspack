const fs = require("node:fs");
const path = require("node:path");
const swc = require("@swc/core");

const runtime = fs.readFileSync(
	path.resolve(__dirname, "../src/runtime/moduleFederationDefaultRuntime.js"),
	"utf-8"
);
const { code: downgradedRuntime } = swc.transformSync(runtime, {
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
	path.resolve(__dirname, "../dist/moduleFederationDefaultRuntime.js"),
	minimizedRuntime.code
);
