import importDx from "./dx-importer";
import { y } from "./module";
import { y as yRe } from "./reexport";

(async () => {
	const dx = await importDx();
	const dz = await import("./dz");
	const dw = await import("./dw");
	console.log(y, yRe, dx, dz, dw);
})();
