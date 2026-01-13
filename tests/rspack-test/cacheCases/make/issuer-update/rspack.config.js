function check_issuer(modules) {
	let available = [];
	let map = {};
	for (const m of modules) {
		map[m.identifier] = m.issuer || true;
	}
	for (const m of modules) {
		if (available.includes(m.identifier)) {
			continue;
		}
		let paths = [];
		let current = m.identifier;
		while (true) {
			if (paths.includes(current)) {
				throw new Error("has cycle issuer");
			}
			paths.push(current);
			const next = map[current];
			if (next == undefined) {
				throw new Error("current issuer module not exist");
			}
			if (next == true) {
				available = available.concat(paths);
				break;
			}
			current = next;
		}
	}
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	cache: {
		type: "persistent"
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.done.tap("PLUGIN", stats => {
					let { modules } = stats.toJson({ modules: true });
					check_issuer(modules);
				});
			}
		}
	]
};
