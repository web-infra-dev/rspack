module.exports = function (content) {
	content += 'exports.lib += "2";\n';
	this.callback(null, content);
};
