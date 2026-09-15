"use strict";

const fs = require("fs");

const findDeterministicBundle = (dir, i) => {
	const re = new RegExp(`^\\d+\\.bundle${i}\\.js$`);
	return fs.readdirSync(dir).find((f) => re.test(f));
};

const bundlePair = (dir, first, i) => {
	if (!first) return [];
	const runtime = `bundle${i}.js`;
	return fs.existsSync(`${dir}/${runtime}`) ? [`./${first}`, `./${runtime}`] : [`./${first}`];
};

module.exports = {
	findBundle(i, options) {
		if (i === 6) {
			return [`bundle${i}.js`];
		}

		const dir = options.output.path;

		if (i === 4 || i === 5) {
			return i === 4
				? [`./use-style-global_js.bundle${i}.js`, `./bundle${i}.js`]
				: bundlePair(dir, findDeterministicBundle(dir, i), i);
		}

		return i === 1 || i === 3
			? bundlePair(dir, findDeterministicBundle(dir, i), i)
			: [`./use-style_js.bundle${i}.js`, `./bundle${i}.js`];
	}
};
