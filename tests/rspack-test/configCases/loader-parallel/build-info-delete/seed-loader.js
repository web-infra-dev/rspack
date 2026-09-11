module.exports = function (content) {
	const { buildInfo } = this._module;
	buildInfo.keep = "kept";
	buildInfo.dropMe = "dropped";
	return content;
};
