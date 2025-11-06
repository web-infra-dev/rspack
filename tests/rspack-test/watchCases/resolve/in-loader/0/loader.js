/** @type {import("@rspack/core").LoaderDefinition} */
module.exports = function () {
	const callback = this.async();
	this.resolve(this.context, "./file", (err, file) => {
		if (err) return callback(err);
		if (!file) return callback(new Error("Resolving failed"));
		// TODO: add dependencies in loaderContext.resolve
		this.addDependency(file);
		this.addMissingDependency(file.replace(".js", ""));

		this.fs.readFile(file, (err, result) => {
			if (err) return callback(err);
			callback(
				null,
				// `export default ${JSON.stringify(result.toString("utf-8").trim())};`
				`export default ${JSON.stringify(result.toString("utf-8").trim())};`
			);
		});
	});
};
