let lastSet;
Object.defineProperty(exports, "value", {
	get: () => require("./module?gs" + __resourceQuery).abc,
	set: v => {
		lastSet = v;
	}
});
exports.getLastSet = () => lastSet;
