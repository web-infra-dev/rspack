module.exports = function (context) {
  this.emitWarning(new Error("Failed to load"));
	return ""
};
