const path = require("path");
const fs = require("fs");

console.log(__dirname);
let abslutePathBindingList = fs
	.readdirSync(".")
	.filter(p => {
		return p.endsWith(".node");
	})
	.map(p => {
		let [_, platform] = p.split(".");
		return {
			platform: platform,
			path: path.join(__dirname, "..", p),
			fileName: p
		};
	});

abslutePathBindingList.forEach(bindingInfo => {
	let npmPath = path.join(__dirname, "../../../npm");
	let packagePath = path.join(npmPath, bindingInfo.platform);
	fs.copyFileSync(
		bindingInfo.path,
		path.join(packagePath, bindingInfo.fileName)
	);
});
