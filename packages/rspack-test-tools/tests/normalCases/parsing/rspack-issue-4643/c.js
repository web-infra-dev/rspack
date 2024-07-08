function c() {
	this.val = "c";
	this.b = "b";
}

c.prototype.value = function () {
	return "c";
};
c.prototype.a = function () {
	return "a";
};

module.exports = c;
