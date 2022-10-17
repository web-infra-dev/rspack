function __invalidate__(dirtyId) {
	function getAccepts(id, module) {
		return module.hot.accepts.filter(({ ids }) => {
			if (typeof ids == "string") {
				return ids === id;
			} else {
				return ids.includes(id);
			}
		});
	}

	function collectModules(id) {
		removed.add(id);
		const module = modules[id];
		const selfAccepts = getAccepts(id, module);
		if (selfAccepts.length > 0) {
			boundaries.add(id);
			for (const accpet of selfAccepts) {
				meta.add(accpet);
			}
		} else {
			for (const m of module.parents.values()) {
				const childAccpets = getAccepts(id, m);
				if (childAccpets.length > 0) {
					for (const accept of childAccpets) {
						boundaries.add(module.id);
						meta.add(accept);
					}
				} else {
					collectModules(m.id);
				}
			}
		}
	}

	runtime.__rspack_require__(dirtyId);

	const boundaries = new Set();
	const meta = new Set();
	const removed = new Set();

	const modules = runtime.moduleCache;
	collectModules(dirtyId);

	for (const mod of removed.values()) {
		delete runtime.moduleCache[modules[mod]];
	}

	for (const id of boundaries.values()) {
		runtime.__rspack_require__(id);
	}

	for (const hot of meta.values()) {
		if (typeof hot.ids === "string") {
			if (hot.accept) {
				const mod = modules[hot.ids];
				hot.accept(mod.exports);
			}
		} else {
			hot.accept(hot.ids.map(id => modules[id].exports));
		}
	}
}

(function () {
	runtime.invalidate = runtime.invalidate || __invalidate__;
})();
