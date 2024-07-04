const path = require("path");
const fs = require("fs");

const {
	any,
	retain,
	each,
	not,
	includedIn,
	matchedInAny,
	matchedWith,
	unify,
	toSet,
	zip
} = require("./math.cjs");
const isBinaryPath = require("./binary-path.cjs");
const {
	recursiveCompare,
	recursiveCompareStrict
} = require("./recursive-compare.cjs");

const WORKSPACE_ROOT = path.resolve(__dirname, "../../");
const RSPACK_TEST = "packages/rspack/tests";
const WEBPACK_TEST = "tests/webpack-test";

const indentLines = s =>
	s
		.split(/\r?\n/)
		.map(s => `\t${s}`)
		.join("\n");
const help = s => `\nhelp:\n${indentLines(s)}`;
const suggestion = s => `\nsuggestion:\n${indentLines(s)}`;

const format = (s, o) => require("prettier").format(s, o);
const CONTENT_COMPARATOR = async (p, a, b) => {
	if (isBinaryPath(p)) {
		return a.equals(b);
	}
	let aa = a.toString().trim();
	let bb = b.toString().trim();
	try {
		const o = {
			filepath: p
		};
		aa = await format(aa, o);
		bb = await format(bb, o);
	} catch (e) { }
	return aa === bb;
};

async function main() {
	const [identical, difference] = await recursiveCompare(
		path.join(WORKSPACE_ROOT, RSPACK_TEST),
		path.join(WORKSPACE_ROOT, WEBPACK_TEST),
		CONTENT_COMPARATOR
	);

	let errored = false;

	// 1. Track tasks that have different content but share the same test name.
	if (difference.size > 0) {
		let excludeList = require("./diff-exclude.cjs");
		let retained = retain(each(not(matchedInAny(excludeList))))(difference);
		if (retained.length > 0) {
			errored = true;
			console.log(
				"1. Mixed test cases: cases below share the same filename with different content on rspack and webpack side.\n"
			);
			console.log(toSet(retained.sort()));
			console.log(
				help(`Due to the historical fact that rspack mixed webpack tests and rspack tests together, 
so this test is served as a helper to decouple these tests.

The following cases share the same name with webpack, however their content are not identical.
This would cause misunderstandings between those tests. This file can be removed after the old tests are no longer coupled with webpack tests.`)
			);
			console.log(
				suggestion(
					`Either ignore these files in the \`${path.relative(process.cwd(), path.join(WORKSPACE_ROOT, "scripts/test/diff-exclude.cjs"))}\` with reason (MUST BE CAUTIOUS) or align it with webpack.\n`
				)
			);
		}
	}

	// 2. Calculate cases that can be safely removed from cases in rspack.
	{
		let maybeIdentical = unify(
			retain(each(matchedWith(/^[^\/]*Cases/)))(identical).map(item => {
				item = item.split("/").slice(0, 3).join("/");
				if (path.extname(item)) {
					return path.dirname(item);
				}
				return item;
			})
		);
		const result = zip(
			maybeIdentical,
			await Promise.all(
				maybeIdentical.map(testPath =>
					recursiveCompareStrict(
						path.join(WORKSPACE_ROOT, RSPACK_TEST, testPath),
						path.join(WORKSPACE_ROOT, WEBPACK_TEST, testPath),
						CONTENT_COMPARATOR
					)
				)
			)
		)
			.filter(([, item]) => item)
			.map(([item, _]) => item);
		if (result.length) {
			console.log(
				"2. Identical test cases: cases below share the identical structure and content in both rspack and webpack.\n"
			);
			console.log(toSet(result.sort()));
			console.log(
				help(`'dist' and 'test.filter.js' are not included in the test.`)
			);
			console.log(
				suggestion(
					`Remove the test cases from rspack and TURN ON webpack test case to suppress this warning.`
				)
			);
		}
	}

	if (errored) {
		process.exit(1);
	}
}

main();
