module.exports = {
	snapshotContent(content) {
		const unusedUsed = /const unusedUsed = (true|false);/.exec(content)?.[1];
		return [
			`exports unused: ${/\\bunused: \\(\\) =>/.test(content)}`,
			`unused used: ${unusedUsed}`
		].join("\n");
	}
};
