import "symlinked-directory-importer";

it("takes the description file from the requested path, not the link target", () => {
	// The link lives in a package without `sideEffects`. Like enhanced-resolve, the
	// target's `sideEffects: false` must not drop this side-effect import.
	expect(globalThis.__symlinkedDirectorySideEffect).toBe("ran");
});
