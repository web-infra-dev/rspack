module.exports = {
	externals: [
		"foo",
		/^raz$/,
		{
			bar: "'bar'",
			baz: "var 'baz'",
			myos: "commonjs os"
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
