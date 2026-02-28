module.exports = function (content) {
	const options = this.getOptions();
	options.files.push(this.resourcePath);
	return content;
};
