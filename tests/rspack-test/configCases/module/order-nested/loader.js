module.exports = function (content) {
	content += 'exports.lib += "0";\n';
	this.callback(null, content);
};
