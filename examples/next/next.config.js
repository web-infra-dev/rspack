process.env.NEXT_RSPACK = "true";
process.env.RSPACK_CONFIG_VALIDATE = "loose-silent";

module.exports = {
	webpack(config) {
		config.plugins.push(c => {
			c.hooks.compilation.tap("next.config.js", compilation => {
				compilation.hooks.processAssets.tap("next.config.js", () => {
					function f() {
						f();
					}
					f();
				});
			});
		});
		return config;
	}
};
