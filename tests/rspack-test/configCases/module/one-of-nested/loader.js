module.exports = function (content) {
	this.callback(null, content.replace("ab", "aab"));
};
