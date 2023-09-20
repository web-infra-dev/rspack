const path = require("path");
const fs = require("fs");

const currentDir = path.resolve(__dirname, "./tree-shaking");
const dirList = fs.readdirSync(currentDir);
const excludeList = ["node_modules"]


const filteredList = dirList.filter((dir) => {
	if (dir.startsWith(".")) {
		return false;
	}
	if (excludeList.includes(dir)) {
		return false;
	}

	return true;
});

console.log(`total: ${filteredList.length}`)

const failedList = filteredList.filter(item => {
	const abPath = path.join(currentDir, item, "snapshot", "snap.diff")
	return fs.existsSync(abPath)
})

console.log(`failed: ${failedList.length}`)
console.log(`passed: ${filteredList.length - failedList.length}`)

