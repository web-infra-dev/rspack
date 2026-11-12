import path from "node:path";

export default {
	validate(stats) {
		for (const child of stats.stats) {
			const modules = Array.from(child.compilation.modules);
			for (const name of ["shared", "target"]) {
				const instances = modules.filter(module =>
					path.basename(module.resource ?? "") === `${name}.module.css`
				);
				expect(instances.length).toBe(1);
			}
			const shared = modules.find(module =>
				path.basename(module.resource ?? "") === "shared.module.css"
			);
			const exportType = child.compilation.options.module.parser.css.exportType;
			expect(shared.identifier().endsWith(`|css-export-type|${exportType}`))
				.toBe(exportType !== undefined);
			const target = modules.find(module =>
				path.basename(module.resource ?? "") === "target.module.css"
			);
			expect(target.identifier()).toMatch(/\|css-export-type\|link$/);
		}
	}
};
