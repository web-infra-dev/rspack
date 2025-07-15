function check_issuer(modules) {
	let available = [];
	const map = {};
	for (const m of modules) {
		map[m.identifier] = m.issuer;
	}
	for (const m of modules) {
		if (available.includes(m.identifier)) {
			continue;
		}
		const paths = [];
		let current = m.identifier;
		while (true) {
			if (!current) {
				available = available.concat(paths);
				break;
			}
			if (paths.includes(current)) {
				throw new Error("has cycle issuer");
			}
			paths.push(current);
			current = map[current];
		}
	}
}

function assert_cycle_module_count(modules, count) {
	let cycle1_count = 0;
	let cycle2_count = 0;
	for (const m of modules) {
		if (/[/\\]cycle1[/\\]/.test(m.identifier)) {
			cycle1_count++;
		}
		if (/[/\\]cycle2[/\\]/.test(m.identifier)) {
			cycle2_count++;
		}
	}
	if (cycle1_count !== count) {
		throw new Error("cycle1 count eq error");
	}
	if (cycle2_count !== count) {
		throw new Error("cycle2 count eq error");
	}
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	plugins: [
		{
			apply(compiler) {
				let time = 0;
				compiler.hooks.done.tap("PLUGIN", stats => {
					const { modules } = stats.toJson({ modules: true });
					if (time == 0) {
						assert_cycle_module_count(modules, 3);
					}
					if (time == 1) {
						assert_cycle_module_count(modules, 3);
					}
					if (time == 2) {
						assert_cycle_module_count(modules, 3);
					}
					if (time == 3) {
						assert_cycle_module_count(modules, 0);
					}
					check_issuer(modules);
					time++;
				});
			}
		}
	]
};
