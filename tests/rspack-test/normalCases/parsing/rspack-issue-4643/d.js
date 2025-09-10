function c() {
	this.val = "c";
}

c.prototype.value = function () {
	return "c";
};

module.exports = {
	c: c
};
