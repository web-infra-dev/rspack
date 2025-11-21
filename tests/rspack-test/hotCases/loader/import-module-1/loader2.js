module.exports = async function (content) {
	if (content.includes("2")) {
		throw "should not throw";
	}
	return content;
};
