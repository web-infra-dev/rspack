const path = require("path");
const fs = require("fs");

console.log(__dirname);
const abslutePathBindingList = fs
	.readdirSync(".")
	.filter(p => {
		return p.endsWith(".node");
	})
	.map(p => {
		const [_, platform] = p.split(".");
		return {
			platform: platform,
			path: path.join(__dirname, "..", p),
			fileName: p
		};
	});

abslutePathBindingList.forEach(bindingInfo => {
	const npmPath = path.join(__dirname, "../../../npm");
	const packagePath = path.join(npmPath, bindingInfo.platform);
	fs.copyFileSync(
		bindingInfo.path,
		path.join(packagePath, bindingInfo.fileName)
	);
});
