const fs = require("fs");
const vm = require("vm");
const path = require("path");

const requireAsync = file => {
	return new Promise((resolve, reject) => {
		fs.readFile(file, { encoding: "utf8" }, function (err, data) {
			const sandbox = {
				module: {
					exports: {}
				},
				require: __non_webpack_require__,
				__dirname: __dirname,
				__filename: __filename
			};
			vm.runInNewContext(data, sandbox);
			sandbox.module.exports.MyLib.then(resolve).catch(reject);
		});
	});
};

it("should work with chunkLoading=async-node and only three or more entrypoint chunks", async () => {
	const chunk = await requireAsync(path.join(__dirname, "async.js"));
	expect(chunk.result).toBe("123");
});
