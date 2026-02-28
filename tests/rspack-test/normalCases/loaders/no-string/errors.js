module.exports = [
	[
		{ moduleName: /\.\/loaders\/no-string\/loader\.js!\.\/loaders\/no-string\/file\.js/ },
		/Buffer, Uint8Array or string expected/
	],
	[
		{ moduleName: /\.\/loaders\/no-string\/loader\.js!\.\/loaders\/no-string\/pitch-loader\.js!\.\/loaders\/no-string\/file\.js/ },
		/Buffer, Uint8Array or string expected/
	]
];
