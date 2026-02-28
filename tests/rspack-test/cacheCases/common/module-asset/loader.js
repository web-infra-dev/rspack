module.exports = function (content) {
	this.emitFile("a.txt", "123");
	return content;
};
