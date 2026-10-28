const RE = /var data = "\d*"/g;

export default {
	checkSteps: false,
	snapshotContent(
		/**@type {string} */
		content
	) {
		return content.replaceAll(RE, "var data = __LAZY_ID__");
	}
};
