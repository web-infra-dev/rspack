// webpack excludes files under `./ctx/node_modules/**` from the context map
// built by `import('./ctx/' + name)` (RequireContextPlugin `hideOriginal`).
// rspack must match. See `alternative_requests` in context_module_factory.rs.
function load(name) {
	return import(/* webpackInclude: /\.stories\.js$/ */ "./ctx/" + name);
}

it("should keep first-party stories in the context map", async () => {
	const { FirstParty } = await load("src/first-party.stories.js");
	expect(FirstParty).toBe("first-party");
});

it("should not pull node_modules files into the context map", async () => {
	await expect(
		load("node_modules/some-dep/src/from-dependency.stories.js")
	).rejects.toThrow(
		"Cannot find module './node_modules/some-dep/src/from-dependency.stories.js'"
	);
});
