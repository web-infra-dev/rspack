const fs = require('fs');
const path = require('path');

// Build-resolved mock identity: the RstestPlugin appends
// `{o: <declaring file>, r: <resolved target | null>}` as the trailing
// argument of every generated `rstest_mock`/`rstest_unmock` call, so the
// @rstest/core runtime keys native (out-of-bundle) mocks by the build's own
// resolution instead of re-deriving it.
it('appends the build-resolved {o, r} identity to rstest_mock/rstest_unmock calls', () => {
	const content = fs.readFileSync(
		path.resolve(__dirname, 'mockResolvedInfo.mjs'),
		'utf-8',
	);

	// 2-arg factory mock of a bundled relative target: `o` is the declaring
	// fixture file, `r` the target's absolute path (both end with the expected
	// file names; separators/roots are platform-specific).
	expect(content).toMatch(
		/rstest_mock\("\.\/src\/dep\.js[^"]*",[\s\S]*?,\s*"\.\/dep\.js",\s*\{"o":"[^"]*fixture\.js","r":"[^"]*dep\.js"\}\)/,
	);

	// 2-arg factory mock of an externalized builtin: `r` is the external
	// request spelling, not a file path.
	expect(content).toMatch(
		/rstest_mock\("node:os[^"]*",[\s\S]*?,\s*"node:os",\s*\{"o":"[^"]*fixture\.js","r":"node:os"\}\)/,
	);

	// Unresolvable package: the mock is still emitted (rstest allows missing
	// modules) and `r` is a json null.
	expect(content).toMatch(
		/"missing-pkg-for-resolved-info",\s*\{"o":"[^"]*fixture\.js","r":null\}\)/,
	);

	// 1-arg auto-mock without a `__mocks__` file: the identity rides the
	// synthetic-target dependency, after the `{ mock: true }` fallback and the
	// request literal — and `r` is the REAL module's path, not a mock target.
	expect(content).toMatch(
		/rstest_mock\("\.\/src\/autoDep\.js[^"]*",\s*\{ mock: true \},\s*"\.\/autoDep\.js",\s*\{"o":"[^"]*fixture\.js","r":"[^"]*autoDep\.js"\}\)/,
	);

	// 1-arg unmock: the identity follows the request on the method call.
	expect(content).toMatch(
		/rstest_unmock\([\s\S]*?,\s*"\.\/unmockDep\.js",\s*\{"o":"[^"]*fixture\.js","r":"[^"]*unmockDep\.js"\}\)/,
	);
});
