module.exports = [
	[
		{ moduleName: /\.\/loaders\/no-string\/loader\.js!\.\/loaders\/no-string\/file\.js/ },
		// CHANGE:
		// /Module build failed: Error: Final loader \(\.\/loaders\/no-string\/loader\.js\) didn't return a Buffer or String/
		/Buffer, Uint8Array or string expected/
	],
	[
		{ moduleName: /\.\/loaders\/no-string\/loader\.js!\.\/loaders\/no-string\/pitch-loader\.js!\.\/loaders\/no-string\/file\.js/ },
		// CHANGE:
		// /Module build failed: Error: Final loader \(\.\/loaders\/no-string\/loader\.js\) didn't return a Buffer or String/
		/Buffer, Uint8Array or string expected/
	]
];
