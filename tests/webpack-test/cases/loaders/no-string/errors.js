module.exports = [
	[
		{ moduleName: /\.\/loaders\/no-string\/loader\.js!\.\/loaders\/no-string\/file\.js/ },
		// CHANGE:
		// /Module build failed: Error: Final loader \(\.\/loaders\/no-string\/loader\.js\) didn't return a Buffer or String/
		/Value is non of these types `String`, `Vec<u8>`, `null`/
	],
	[
		{ moduleName: /\.\/loaders\/no-string\/loader\.js!\.\/loaders\/no-string\/pitch-loader\.js!\.\/loaders\/no-string\/file\.js/ },
		// CHANGE:
		// /Module build failed: Error: Final loader \(\.\/loaders\/no-string\/loader\.js\) didn't return a Buffer or String/
		/Value is non of these types `String`, `Vec<u8>`, `null`/
	]
];
