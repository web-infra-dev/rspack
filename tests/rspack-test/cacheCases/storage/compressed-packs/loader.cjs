module.exports = function (source) {
	return `/*${"synthetic cache compression fixture\n".repeat(16 * 1024)}*/\n${source}`;
};
