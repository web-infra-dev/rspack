module.exports = {
	externals: [
		"foo",
		/^raz$/,
		{
			bar: "'bar'",
			baz: "var 'baz'",
			myos: "commonjs os"
		}
	],
	externalsPresets: {
		node: false
	}
};
