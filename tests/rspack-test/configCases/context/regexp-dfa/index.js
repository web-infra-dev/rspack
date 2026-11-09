it("prunes unrelated directories for an anchored context regexp", () => {
	const context = require.context("./pruning", true, /^\.\/pages\/.*\.js$/);
	expect(context.keys()).toEqual(["./pages/entry.js"]);
});

it("matches alternative requests for index files, extensions and module roots", () => {
	const main = require.context("./aliases", true, /^\.\/pages$/);
	const extensionless = require.context("./aliases", true, /^\.\/pages\/item$/);
	const bare = require.context("./aliases", true, /^example$/);
	expect(main.keys()).toEqual(["./pages"]);
	expect(extensionless.keys()).toEqual(["./pages/item"]);
	expect(bare.keys()).toEqual(["example"]);
});

it("keeps JS regexp semantics for backreferences and lookbehind", () => {
	const backreference = require.context("./fallback", true, /^\.\/(\w+)-\1\.js$/);
	const lookbehind = require.context("./fallback", true, /(?<=word-)other\.js$/);
	expect(backreference.keys()).toEqual(["./word-word.js"]);
	expect(lookbehind.keys()).toEqual(["./word-other.js"]);
});

it("preserves JS word classes and unanchored matching", () => {
	const word = require.context("./fallback", true, /^\.\/\w+\.js$/);
	const unanchored = require.context("./fallback", true, /word-.*\.js/);
	expect(word.keys()).toEqual(["./word.js"]);
	expect(unanchored.keys().sort()).toEqual(["./word-other.js", "./word-word.js"]);
});
