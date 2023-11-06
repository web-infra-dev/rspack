const path = require("path");
const fs = require("fs");

const currentDir = path.resolve(__dirname, "./tree-shaking");
const dirList = fs.readdirSync(currentDir);
const excludeList = ["node_modules"];

const filteredList = dirList.filter((dir) => {
	if (dir.startsWith(".")) {
		return false;
	}
	if (excludeList.includes(dir)) {
		return false;
	}

	return true;
});

console.log(`total: ${filteredList.length}`);

const falsePositiveMap = {
	"cjs-export-computed-property":
		"This one is false positive because webpack will not counted a esm export as unsed in an entry module, the previous implementation follows the esbuild behavior , see https://gist.github.com/IWANABETHATGUY/b41d0f80a558580010276a44b310a473",
	basic: "align webpack unused binding behavior",
	"context-module-elimated": "align webpack unused binding behavior",
	"rollup-unused-called-import": "align webpack unused binding behavior",
	"var-function-expr": "align webpack unused binding behavior",
	"webpack-innergraph-no-side-effects": "align webpack unused binding behavior",
	"side-effects-export-default-expr": "align webpack unused binding behavior",
	"webpack-innergraph-circular": "align webpack unused binding behavior https://github.com/web-infra-dev/rspack/pull/4463/files",
	"static-class": "align webpack unused binding behavior https://github.com/web-infra-dev/rspack/pull/4463/files",
	"webpack-inner-graph-export-default-named": "align webpack unused binding behavior https://github.com/web-infra-dev/rspack/pull/4463/files"
};

const normalizedList = filteredList.map((item) => {
	const abPath = path.join(currentDir, item, "snapshot", "snap.diff");
	let status = fs.existsSync(abPath);
	return {
		name: item,
		reason: falsePositiveMap[item],
		passed: !status,
	};
});

let successedCount = normalizedList.filter((item) => {
	return item.passed || !!item.reason;
}).length;

let fasePositiveCases = normalizedList
	.filter((item) => {
		return !!item.reason;
	})
	.map((item) => {
		return `${item.name}: ${item.reason}`;
	});
let failedCases = normalizedList
	.filter((item) => !item.passed && !item.reason)
	.map((item) => {
		return item.name;
	});

console.log(`failed: ${filteredList.length - successedCount}`);
console.log(`passed: ${successedCount}`);
console.log(`fasePositiveCases: ${fasePositiveCases.length}\n`, fasePositiveCases);
console.log(`failedCases: ${failedCases.length}\n`, failedCases);
