this.aaa = function () {
	return 1;
};
this.aaa.bbb = function () {
	return 2;
};
Object.defineProperty(this, "ccc", {
	value: function () {
		return 3;
	}
});
Object.defineProperty(this.ccc, "ddd", {
	value: function () {
		return 4;
	}
});
