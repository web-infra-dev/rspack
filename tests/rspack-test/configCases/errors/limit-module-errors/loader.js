module.exports = function (content) {
	for (let i = 0; i < 101; i++) {
		this.emitError(new Error(`emitted error ${i}`));
	}
	this.emitWarning(new Error('emitted warning'));
	return content;
};
