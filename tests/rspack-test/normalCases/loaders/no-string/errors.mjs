export default [
	[
		{ moduleName: /\.\/loaders\/no-string\/loader\.mjs!\.\/loaders\/no-string\/file\.js/ },
		/Buffer, Uint8Array or string expected/
	],
	[
		{ moduleName: /\.\/loaders\/no-string\/loader\.mjs!\.\/loaders\/no-string\/pitch-loader\.mjs!\.\/loaders\/no-string\/file\.js/ },
		/Buffer, Uint8Array or string expected/
	]
];
