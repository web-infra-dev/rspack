module.exports = function (source) {
	this.emitWarning(new Error("warning from stats-artifact-fallback-warning"));
	return source;
};
