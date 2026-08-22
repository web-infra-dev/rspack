it("should load an async extracted css chunk in rspack runtime mode without HMR", async () => {
	// Regression test: extractCssLoadStylesheet weakly reads HMR_MINI_CSS_FILENAMES so it
	// can prefer a fresh HMR-updated filename over the chunk filename function. That weak
	// read must still force a (possibly unused) lexical declaration of the global under
	// runtimeMode: "rspack", or referencing it here - with no HMR runtime module present at
	// all - throws a ReferenceError instead of just falling back to the chunk filename.
	await import(/* webpackChunkName: "lazy" */ "./lazy.css");

	const link = document.head._children.find(function (child) {
		return child._type === "link";
	});
	expect(link).toBeTruthy();
	expect(link.href).toContain("lazy.css");
});
