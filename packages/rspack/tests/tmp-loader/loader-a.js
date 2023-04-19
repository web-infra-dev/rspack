module.exports = function (content) {
	console.log("a", this.getOptions());
	return content + `\nconsole.log("a")`;
};
