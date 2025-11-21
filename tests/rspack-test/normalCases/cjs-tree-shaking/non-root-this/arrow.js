const F = () => {
	this.test1 = true;
	Object.defineProperty(this, "test2", {
		value: true
	});
	return this;
};
exports.fff = F();
