const shorthand = "shorthand";
let setterValue;
module.exports = {
	keyValue: "key-value",
	shorthand,
	"quoted-key": "quoted",
	unused: "unused",
	method() {
		return "method";
	},
	get getter() {
		return "getter";
	},
	set setter(value) {
		setterValue = value;
	},
	readSetter: () => setterValue,
	async asyncMethod() {
		return "async";
	},
	*generatorMethod() {
		yield "generator";
	},
	toString: "to-string",
	canMangle: __webpack_exports_info__.keyValue.canMangle
};
