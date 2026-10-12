export default {
	validate(stats) {
		for (const child of stats.stats) {
			const modules = Array.from(child.compilation.modules);
			for (const name of ["shared", "target"]) {
				const instances = modules.filter(module =>
					module.resource?.endsWith(`/${name}.module.css`)
				);
				expect(instances.length).toBe(1);
			}
			const shared = modules.find(module =>
				module.resource?.endsWith("/shared.module.css")
			);
			const exportType = child.compilation.options.module.parser.css.exportType;
			expect(shared.identifier().endsWith(`|css-export-type|${exportType}`))
				.toBe(exportType !== undefined);
			const target = modules.find(module =>
				module.resource?.endsWith("/target.module.css")
			);
			expect(target.identifier()).toMatch(/\|css-export-type\|link$/);
		}
	}
};
