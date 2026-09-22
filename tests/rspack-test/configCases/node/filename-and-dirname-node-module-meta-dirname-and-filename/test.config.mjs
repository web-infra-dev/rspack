import path from "node:path";
import { pathToFileURL } from "node:url";

let counter = 0;

export default {
	moduleScope(scope, _stats, options) {
		const bundleFilename = path.join(
			options.output.path,
			`bundle${counter++}.mjs`
		);
		scope.custom = {
			url: pathToFileURL(bundleFilename).toString(),
			dirname: path.dirname(bundleFilename),
			filename: bundleFilename
		};
	}
};
