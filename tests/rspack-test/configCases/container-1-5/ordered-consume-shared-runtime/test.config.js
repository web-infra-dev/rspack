module.exports = {
	findBundle: function (i) {
		switch (i) {
			// shared runtime, scalar entry first
			case 0:
				return [
					"./shared-scalar-first/runtime.js",
					"./shared-scalar-first/scalar.js",
					"./shared-scalar-first/ordered.js",
				];
			// shared runtime, ordered entry first
			case 1:
				return [
					"./shared-ordered-first/runtime.js",
					"./shared-ordered-first/ordered.js",
					"./shared-ordered-first/scalar.js",
				];
			// separate runtimes
			default:
				return ["./separate/scalar.js", "./separate/ordered.js"];
		}
	},
};
