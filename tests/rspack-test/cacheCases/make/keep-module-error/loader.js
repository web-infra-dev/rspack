module.exports = function (code) {
	this.emitError(new Error("LoaderError"));
	return code;
};
