module.exports = {
	snapshotContent(content) {
		const wrapperStart = content.indexOf(
			"// wrapper to provide named exports for ESM."
		);

		if (wrapperStart < 0) {
			throw new Error("Expected the Commander ESM wrapper in the output");
		}

		return content.slice(wrapperStart);
	}
};
