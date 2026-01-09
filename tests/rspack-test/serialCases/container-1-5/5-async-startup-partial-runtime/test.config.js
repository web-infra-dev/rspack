/** @type {import('@rspack/test-tools').TConfigCaseConfig} */
module.exports = {
	findBundle: function (i, options) {
		const uniqueName = (options.output && options.output.uniqueName) || '';
		if (uniqueName.includes('0-container-full')) return;
		return './plain.js';
	}
};
