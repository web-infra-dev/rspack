it("should allow to access __webpack_get_script_filename__ ", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	const oldFn = __webpack_get_script_filename__;
	__webpack_get_script_filename__ = chunk => {
		const filename = oldFn(chunk);
		return filename + ".changed";
	};
	import("./test.js").then(
		() => {
			done.fail("Loading chunk should fail");
		},
		err => {
			expect(err.code).toBe("ENOENT");
			expect(err.path).toMatch(/\.js\.changed$/);
			done();
		}
	);
}));
