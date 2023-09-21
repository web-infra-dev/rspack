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

const falsePositiveList = [
	'cjs-export-computed-property', // This one is false positive because webpack will not counted a esm export as unsed in an entry module, the previous implementation follows the esbuild behavior , see https://gist.github.com/IWANABETHATGUY/b41d0f80a558580010276a44b310a473
]
const failedList = filteredList.filter(item => {
	if (falsePositiveList.includes(item)) {
		return false;
	}
	const abPath = path.join(currentDir, item, "snapshot", "snap.diff")
	return fs.existsSync(abPath)
})

console.log(`failed: ${failedList.length}`)
console.log(`passed: ${filteredList.length - failedList.length}`)
console.log('failed list:\n', failedList)
