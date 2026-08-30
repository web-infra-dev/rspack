module.exports = {
	used: "used",
	unusedMethod() {
		return super.value;
	},
	get unusedGetter() {
		return super.value;
	},
	set unusedSetter(value) {
		super.value = value;
	},
	async unusedAsyncMethod() {
		return super.value;
	},
	*unusedGeneratorMethod() {
		yield super.value;
	},
	async *unusedAsyncGeneratorMethod() {
		yield super.value;
	}
};
