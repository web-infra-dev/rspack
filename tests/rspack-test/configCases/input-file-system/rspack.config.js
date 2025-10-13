const assert = require("node:assert");

let readFileCalled = false;
let stateCalled = false;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		index: "./virtual_index.js"
	},
	plugins: [
		{
			apply: compiler => {
				compiler.hooks.afterCompile.tap("SimpleInputFileSystem", () => {
					assert(readFileCalled, "readFile should be called");
					assert(stateCalled, "stat should be called");
				});
				compiler.hooks.beforeCompile.tap("SimpleInputFileSystem", () => {
					// simple file system just works for test case
					compiler.inputFileSystem = {
						readFile(p, cb) {
							readFileCalled = true;
							cb(
								null,
								`
								require("./disk.js");
								it("should read file simple file",()=>{
								  expect(1).toBe(1);
							  })`
							);
						},
						stat(p, cb) {
							stateCalled = true;
							cb(null, {
								isFile() {
									return true;
								},
								isDirectory() {
									return false;
								},
								isSymbolicLink() {
									return false;
								},
								atimeMs: 1749025843289.6816,
								mtimeMs: 1749025842638.571,
								ctimeMs: 1749025842638.571,
								birthtimeMs: 1749025385767.096,
								size: 1000
							});
						}
					};
				});
			}
		}
	],
	experiments: {
		useInputFileSystem: [/virtual_.*\.js/]
	}
};
