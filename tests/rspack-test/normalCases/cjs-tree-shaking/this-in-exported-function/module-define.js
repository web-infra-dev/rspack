Object.defineProperty(exports, "a", {
	value: function a() {
		return this.b();
	}
});

Object.defineProperty(exports, "b", {
	value: function b() {
		return "b";
	}
});

Object.defineProperty(exports, "c", {
	get() {
		return this.b();
	}
});

const enumerable = { enumerable: true };

Object.defineProperty(exports, "d", {
	...enumerable,
	set(value) {
		this.received = this.b() + value;
	}
});

exports.usedExports = __webpack_exports_info__.usedExports;
