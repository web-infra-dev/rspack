module.exports = function (content) {
	content += 'exports.lib += "1";\n';
	this.callback(null, content);
};
