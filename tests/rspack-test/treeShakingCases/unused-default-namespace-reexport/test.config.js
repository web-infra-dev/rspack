module.exports = {
	snapshotContent(content) {
		return `empty pure expression: ${content.includes(
			"/* unused pure expression or super */ null && ()"
		)}`;
	}
};
