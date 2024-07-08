const fs = require("fs");
const path = require("path");
const terser = require("terser");

const runtime = fs.readFileSync(
	path.resolve(__dirname, "../src/container/default.runtime.js")
);

const output = terser.minify_sync(runtime.toString(), {
	compress: false,
	mangle: false
});

fs.writeFileSync(
	path.resolve(__dirname, "../dist/container/default.runtime.js"),
	output.code
);
