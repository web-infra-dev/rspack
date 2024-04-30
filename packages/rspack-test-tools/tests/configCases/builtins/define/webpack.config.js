const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: ["./index.js"]
	},
	plugins: [
		new rspack.DefinePlugin({
			TRUE: true,
			FALSE: false,
			TRUE_STRING: "true",
			FALSE_STRING: "false",
			NUMBER_ADD: "3 + 2",
			NULL: "null",
			UNDEFINED: undefined,
			UNDEFINED_STRING: "undefined",
			FUNCTION: "(function(num) { return num + 1 })",
			NUMBER: "100.05",
			ZERO: "0",
			ONE: "1",
			BIGINT: "BigInt(10000)",
			BIGINT2: "100000000000n",
			POSITIVE_ZERO: "+0",
			NEGATIVE_ZERO: "-0",
			POSITIVE_NUMBER: "+100.25",
			NEGATIVE_NUMBER: "-100.25",
			STRING: '"string"',
			EMPTY_STRING: '""',
			REGEXP: "/abc/i",
			OBJECT:
				'{UNDEFINED: undefined, REGEXP: /def/i, STR: "string", OBJ: { NUM: 1}}',
			"P1.P2.P3": "301",
			"P1.P2.P4": '"302"',
			P1: "303",
			"P1.P2": "304",
			ARRAY: '[300, ["six"]]',
			DO_NOT_CONVERTED: "DO_NOT_CONVERTED_TAG",
			DO_NOT_CONVERTED2: "DO_NOT_CONVERTED_TAG",
			DO_NOT_CONVERTED3: "DO_NOT_CONVERTED_TAG",
			DO_NOT_CONVERTED4: "DO_NOT_CONVERTED_TAG",
			DO_NOT_CONVERTED5: "DO_NOT_CONVERTED_TAG",
			DO_NOT_CONVERTED6: "DO_NOT_CONVERTED_TAG",
			DO_NOT_CONVERTED7: "DO_NOT_CONVERTED_TAG",
			DO_NOT_CONVERTED8: "DO_NOT_CONVERTED_TAG",
			DO_NOT_CONVERTED9: "DO_NOT_CONVERTED_TAG",
			IN_BLOCK: "SHOULD_BE_CONVERTED_IN_UNDEFINED_BLOCK",
			"M1.M2.M3": "{}",
			SHOULD_CONVERTED: "205",
			CONVERTED_TO_MEMBER: "A1.A2.A3"
		})
	]
};
