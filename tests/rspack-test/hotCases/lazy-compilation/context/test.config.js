const RE = /var data = "\d*"/g;

module.exports = {
	checkSteps: false,
	snapshotContent(
		/**@type {string} */
		content
	) {
		return content.replaceAll(RE, "var data = __LAZY_ID__");
	}
};
