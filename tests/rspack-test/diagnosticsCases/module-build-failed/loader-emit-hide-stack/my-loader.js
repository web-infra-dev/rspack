module.exports = function (context) {
	let e;
	e = new Error("Failed to load");
	e.hideStack = true;
	this.emitError(e);

	e = new Error("Failed to load");
	e.hideStack = true;
	this.emitWarning(e);
	return ""
};
