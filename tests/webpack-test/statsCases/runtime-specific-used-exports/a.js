import importDx from "./dx-importer";
import { x } from "./module";
import { x as xRe } from "./reexport";

(async () => {
	const dx = await importDx();
	const dy = await import("./dy");
	const dw = await import("./dw");
	console.log(x, xRe, dx, dy, dw);
})();
