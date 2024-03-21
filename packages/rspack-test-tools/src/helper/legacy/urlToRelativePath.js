// @ts-nocheck
module.exports = function urlToRelativePath(url) {
	if (url.startsWith("https://test.cases/path/")) url = url.slice(24);
	return `./${url}`;
};
