module.exports = {
	builtins: {
		define: {
			EXPECTED: "Array.isArray",
			EXPECTED2: "path.resolve",
			EXPECTED3: "globalThis"
		}
	},
	externals: [
		"foo",
		/^raz$/,
		{
			bar: "'bar'",
			baz: "var 'baz'",
			myos: "commonjs os",
			external: ["Array", "isArray"],
			external2: ["commonjs path", "resolve"],
			external3: ["var globalThis"]
		},
		function ({ request }, callback) {
			if (request === "fn") {
				return callback(null, "'fn'");
			}
			callback();
		},
		async function ({ request }) {
			if (request === "asyncFn") {
				return "'asyncFn'";
			}
		}
	],
	externalsPresets: {
		node: false
	}
};
