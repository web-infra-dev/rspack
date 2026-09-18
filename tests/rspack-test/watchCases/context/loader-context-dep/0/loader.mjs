import path from "node:path";
const directory = path.resolve(import.meta.dirname, "directory");

/** @type {import("@rspack/core").LoaderDefinition} */
export default function () {
	this.addContextDependency(directory);
	const callback = this.async();
	this.fs.readdir(directory, (err, files) => {
		if (err) return callback(err);
		files.sort();
		callback(null, `module.exports = ${JSON.stringify(files)};`);
	});
};
