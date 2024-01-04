const { rspack } = require("@rspack/core");
const { createFsFromVolume, Volume } = require("memfs");
const config = require("./rspack.config");

const fs = createFsFromVolume(
	Volume.fromJSON({
		"/app/main.js": 'console.log("from mem-fs")',
		"/app1/index.html": "<!DOCTYPE html><title>It works!</title>It works!"
	})
);
const compiler = rspack({
	...config,
	context: "/app1",
	entry: "/app/main.js",
	output: { path: "/dist", clean: true }
});

compiler.inputFileSystem = fs;
compiler.intermediateFileSystem = fs;
compiler.outputFileSystem = fs;

compiler.run((err, stats) => {
	if (err) {
		console.error(err);
	}
	if (stats) {
		console.log(stats.toString({ colors: true }));
	}
});
