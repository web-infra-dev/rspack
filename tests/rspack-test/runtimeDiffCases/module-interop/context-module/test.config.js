/** @type {import("@rspack/test-tools").TDiffCaseConfig} */
module.exports = {
	modules: true,
	runtimeModules: false,
	// TODO: remove renameModule and replacements once webpack merges https://github.com/webpack/webpack/pull/19903
	renameModule(moduleName) {
		const matches = moduleName.match(/context-module\/src\/namespace-object-lazy\/dir-(?:cjs|esm|mixed)\|async-weak\|.*\|referencedExports: /);
		if (matches) {
			return moduleName.replace("|referencedExports: ", "");
		}
		return moduleName;
	},
	replacements: [
		{
			from: /"\.\/src\/namespace-object-lazy\/dir-(?:cjs|esm|mixed) async-weak recursive .* referencedExports: "/g,
			to: (substring) => substring.replace(" referencedExports: ", ""),
		}
	]
};
