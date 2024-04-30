module.exports = function () {
	function c() {
		this.val = "c";
	}
	c.prototype.value = function () {
		return "c";
	};
	return c;
};
