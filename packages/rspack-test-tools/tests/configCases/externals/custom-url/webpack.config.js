module.exports = {
	externals: [
		function ({ request, dependencyType }, callback) {
			if (/^(\/\/|custom?:\/\/)/.test(request)) {
				if (dependencyType === "css-import")
					return callback(null, `css-import ${request}`);
				if (dependencyType === "url") return callback(null, `asset ${request}`);
				return callback(null, `var '${request}'`);
			}
			return callback();
		}
	],
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false,
			}
		}
	}
};
