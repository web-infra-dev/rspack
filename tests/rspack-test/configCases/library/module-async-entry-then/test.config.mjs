import fs from "node:fs";
import path from "node:path";

export default {
	findBundle(index, options) {
		if (options.name.startsWith("chunk-format-")) return [];
		return `./${options.name}/main.mjs`;
	},
	validate(stats, stderr, options) {
		for (const config of options) {
			if (!config.name.startsWith("chunk-format-")) continue;
			// Other chunk formats still need the library plugin to await the entry.
			const source = fs.readFileSync(
				path.join(config.output.path, config.name, "library.mjs"),
				"utf8"
			);
			expect(source.match(/(\b\w+) = await \1;/g)).toHaveLength(1);
		}
	}
};
