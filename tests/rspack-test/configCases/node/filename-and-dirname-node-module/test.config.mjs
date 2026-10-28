import path from "node:path";
import { pathToFileURL } from "node:url";

export default {
	moduleScope(scope, stats, options) {
		scope.custom = {
			url: pathToFileURL(path.join(options.output.path, "bundle0.mjs"))
		};
	}
};
