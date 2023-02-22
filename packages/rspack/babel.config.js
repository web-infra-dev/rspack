const MIN_BABEL_VERSION = 7;

module.exports = api => {
	api.assertVersion(MIN_BABEL_VERSION);
	api.cache(true);

	return {
		presets: [
			[
				"@babel/preset-env",
				{
					exclude:
						process.env.NODE_ENV === "test" ? [] : ["proposal-dynamic-import"],
					targets: {
						node: "14.15.0"
					}
				}
			]
		]
	};
};
