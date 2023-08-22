const assert = require("assert");

let last;

module.exports = {
	plugins: [
		function (compiler) {
			compiler.hooks.done.tap("test", stats => {
				last = stats;
			});
		},
		function (compiler) {
			compiler.hooks.beforeCompile.tap("test", () => {
				if (last) {
					const { hash } = last.toJson({ all: false, hash: true });
					assert(hash !== undefined);
				}
			});
		}
	]
};
