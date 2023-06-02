function getNormalizedFilterName(flag, testName) {
	switch (flag) {
		case false:
			return testName;
		case -1:
			return `WillNotSupport${testName}`;
		default:
			return "";
	}
}

module.exports = {
	getNormalizedFilterName,
};
