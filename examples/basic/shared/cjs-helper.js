// CommonJS helper module to test PURE annotations for CommonJS requires
module.exports = {
	helperFunction: function (input) {
		return `Helper processed: ${input}`;
	},

	HELPER_CONSTANT: "CommonJS_HELPER",

	createHelper: function (name) {
		return {
			name: name,
			process: function (data) {
				return `${name} processed: ${data}`;
			}
		};
	}
};

module.exports.version = "1.0.0";
