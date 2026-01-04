/** @type {import('@rspack/test-tools').TConfigCaseConfig} */
module.exports = {
	writeStatsJson: true,
	findBundle: function (i, options) {
		const uniqueName = options.output && options.output.uniqueName || "";
		const isModule = Boolean(options.experiments && options.experiments.outputModule);
		const isRemote = uniqueName.includes("0-container-full");

		// Skip executing the helper remotes we build in this case.
		if (isRemote) return;

		return isModule ? "./module/main.mjs" : "./main.js";
	}
};
