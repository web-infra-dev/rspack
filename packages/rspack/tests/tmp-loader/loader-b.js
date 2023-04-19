module.exports = function (content) {
	console.log("b", this.getOptions());
	return content + `\nconsole.log("b")`;
};
