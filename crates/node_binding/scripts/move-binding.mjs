import path from "node:path";
import fs from "node:fs";

const abslutePathBindingList = fs
	.readdirSync(".")
	.filter(p => {
		return p.endsWith(".node");
	})
	.map(p => {
		const [_, platform] = p.split(".");
		return {
			platform: platform,
			path: path.join(import.meta.dirname, "..", p),
			fileName: p
		};
	});

abslutePathBindingList.forEach(bindingInfo => {
	const npmPath = path.join(import.meta.dirname, "../../../npm");
	const packagePath = path.join(npmPath, bindingInfo.platform);
	fs.copyFileSync(
		bindingInfo.path,
		path.join(packagePath, bindingInfo.fileName)
	);
});
