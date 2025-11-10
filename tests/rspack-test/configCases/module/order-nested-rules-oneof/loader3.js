module.exports = function (content) {
	content += 'exports.lib += "3";\n';
	this.callback(null, content);
};
