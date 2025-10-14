import './change'

const fs = __non_webpack_require__('fs')
const path = __non_webpack_require__('path')

it("should have correct order", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	const content = fs.readFileSync(path.resolve(__dirname, './bundle.css')).toString()
	expect(content.replaceAll('\n', '').trim()).toBe('.a{}.b{}')

	module.hot.accept("./change", () => {

	});
	NEXT(
		require("@rspack/test-tools/helper/legacy/update")(done, true, () => {
			const content = fs.readFileSync(path.resolve(__dirname, './bundle.css')).toString()
			expect(content.replaceAll('\n', '').trim()).toBe('.b{}.a{}')
			done()
		})
	);
}));

