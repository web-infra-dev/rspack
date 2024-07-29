/** @type {import("../../../../dist").TDiffCaseConfig} */
module.exports = {
	renameModule: (raw) => {
		console.log(raw);
		console.log(raw.split("|").slice(0, -1).join('|'));
		return raw.split("|").slice(0, -1).join('|');
	},
	modules: true,
	files: [
		'shared.js',
		'a.js',
		'b.js',
	],
};
