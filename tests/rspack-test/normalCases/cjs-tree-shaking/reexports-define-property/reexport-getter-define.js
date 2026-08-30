Object.defineProperty(exports, "getter1", {
	enumerable: true,
	get: () => require("./module?dg1" + __resourceQuery)
});
Object.defineProperty(module.exports, "getter2", {
	enumerable: true,
	get: function () {
		return require("./module?dg2" + __resourceQuery).abc;
	}
});
Object.defineProperty(this, "getter3", {
	enumerable: true,
	get() {
		return require("./module?dg3" + __resourceQuery);
	}
});
