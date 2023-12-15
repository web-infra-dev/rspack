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
	"webpack-innergraph-circular":
		"align webpack unused binding behavior https://github.com/web-infra-dev/rspack/pull/4463/files",
	"static-class":
		"align webpack unused binding behavior https://github.com/web-infra-dev/rspack/pull/4463/files",
	"webpack-inner-graph-export-default-named":
		"align webpack unused binding behavior https://github.com/web-infra-dev/rspack/pull/4463/files",
	"class-extend":
		"align webpack unused binding behavior https://github.com/web-infra-dev/rspack/pull/4481/files",
	export_star:
		"same as webpack https://gist.github.com/IWANABETHATGUY/1ee8aa4c2889a9246d19d7be0ac75bb7",
	"issue-4637":
		"align webpack https://github.com/web-infra-dev/rspack/pull/4637/files#diff-d434486532fb1507da93a26ce108dca465337b6af0ee86b4ab94fd788810d288",
	"named-export-decl-with-src-eval":
		"align to webpack: https://github.com/web-infra-dev/rspack/pull/4629/files#r1393574682",
	"rollup-unused-var":
		"align to webpack,  https://github.com/web-infra-dev/rspack/pull/4629/files#r1393575194",
	"ts-target-es5":
		"align to webpack, https://github.com/web-infra-dev/rspack/pull/4629/files#r1392268704",
	"webpack-reexport-namespace-and-default":
		"align to webapck, https://github.com/web-infra-dev/rspack/pull/4629/files#r1393576913",
	bb: "update dep",
	"cjs-tree-shaking-basic": "update dep",
	"cyclic-reference-export-all":
		"redundant `usePlatform: function() { return usePlatform;`",
	"export-imported-import-all-as": "update dep",
	"import-var-assign-side-effects": "update dep",
	"nested-import-3": "update dep",
	"nested-import-4": "update dep",
	"prune-bailout-module": "update dep",
	pure_comments_magic_comments: "update dep",
	"reexport-all-as": "update dep",
	reexport_entry_elimination: "update dep",
	"side-effects-analyzed": "update dep",
	"side-effects-prune": "update dep",
	"side-effects-two": "update dep",
	"export-star-chain": "update dep",
	"import-export-all-as-a-empty-module": "update dep",
	"import-star-as-and-export": "update dep",
	"module-rule-side-effects2": "align with webpack",
	"react-redux-like": "update dep",
	"webpack-side-effects-all-used": "update dep",
	"webpack-side-effects-simple-unused": "update dep",
	"reexport-all-as-multi-level-nested": "update dep",
	"conflicted_name_by_re_export_all_should_be_hidden": "remove es runtime, align webpack, see #4995",
  'explicit_named_export_higher_priority_1': 'remove es runtime, align webpack, see #4995',
  'explicit_named_export_higher_priority_2': 'remove es runtime, align webpack, see #4995',
	"namespace-access-var-decl-rhs": "update dep",
  'export-named-decl-as': "remove es runtime, align webpack, see #4995",
  'export_star2': 'remove es runtime, align webpack, see #4995',
  'export_star_conflict_export_no_error': 'remove es runtime, align webpack, see #4995',
  'inherit_export_map_should_lookup_in_dfs_order':'remove es runtime, align webpack, see #4995',

  'reexport_default_as': "remove es runtime, align webpack, see #4995",
  'rename-export-from-import': 'remove es runtime, align webpack, see #4995',
  'side-effects-flagged-only': 'remove es runtime, align webpack, see #4995',
  'simple-namespace-access': 'remove es runtime, align webpack, see #4995'
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
console.log(
	`fasePositiveCases: ${fasePositiveCases.length}\n`,
	fasePositiveCases,
);
console.log(`failedCases: ${failedCases.length}\n`, failedCases);
