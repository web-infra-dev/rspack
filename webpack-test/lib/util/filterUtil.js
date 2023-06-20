function getNormalizedFilterName(flag, testName) {
	switch (flag) {
		case false:
			return testName;
		case -1:
			return `WillNotSupport${testName}`;
		default:
			if (typeof flag === 'string') {
				return flag
			}
			return "";
	}
}

module.exports = {
	getNormalizedFilterName,
};
