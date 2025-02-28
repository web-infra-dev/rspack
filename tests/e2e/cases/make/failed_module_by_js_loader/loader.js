module.exports = function (content) {
	if (content.includes("error")) {
		throw new Error("loader transform error");
	}
	return content;
};
