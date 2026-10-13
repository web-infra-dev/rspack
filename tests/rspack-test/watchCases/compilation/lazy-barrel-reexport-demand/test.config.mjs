import fs from "node:fs";
import path from "node:path";

export default {
	findBundle(_index, options) {
		return options.output.filename;
	},
	moduleScope(scope, _stats, options) {
		if (!scope.__filename) return scope;
		if (Array.isArray(options)) {
			options = options.find(
				option => option.output.filename === path.basename(scope.__filename)
			);
		}
		const source = fs.readFileSync(
			path.join(options.output.path, options.output.filename),
			"utf8"
		);
		scope.BARREL_FACTORIES = ["index", "named"].map(name => {
			const marker = `"./lib/${name}.js"(`;
			const start = source.indexOf(marker);
			if (start === -1) throw new Error(`Missing barrel factory: ${name}`);
			const end = source.indexOf("\n},", start);
			if (end === -1) throw new Error(`Missing factory end: ${name}`);
			return source.slice(start, end + 2);
		});
		scope.CONFIG_KEY = `${options.cache}-${options.incremental}`;
		return scope;
	}
};
