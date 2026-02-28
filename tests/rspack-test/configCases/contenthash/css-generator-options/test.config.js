const findOutputFiles = require("@rspack/test-tools/helper/legacy/findOutputFiles");

const allCss = new Set();
const allBundles = new Set();

module.exports = {
	findBundle: function (i, options) {
		if (i === 3) {
			return [];
		}
		const bundle = findOutputFiles(options, new RegExp(`^bundle${i}`))[0];
		const async = findOutputFiles(options, /\.js/, `css${i}`);
		allBundles.add(/\.([^.]+)\./.exec(bundle)[1]);
		const css = findOutputFiles(options, /^.*\.[^.]*\.css$/, `css${i}`)[0];
		allCss.add(css);

		return [`./css${i}/${async}`, `./${bundle}`];
	},
	afterExecute: () => {
		expect(allBundles.size).toBe(6);
		expect(allCss.size).toBe(6);
	}
};
