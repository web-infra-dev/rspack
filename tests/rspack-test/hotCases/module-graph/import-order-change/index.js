import './change'

const fs = __non_webpack_require__('fs')
const path = __non_webpack_require__('path')

it("should have correct order", done => {
	const content = fs.readFileSync(path.resolve(__dirname, './bundle.css')).toString()
	expect(content.replaceAll('\n', '').trim()).toBe('.a{}.b{}')

	module.hot.accept("./change", () => {
		
	});
	NEXT(
		require("../../update")(done, true, () => {
			const content = fs.readFileSync(path.resolve(__dirname, './bundle.css')).toString()
			expect(content.replaceAll('\n', '').trim()).toBe('.b{}.a{}')
			done()
		})
	);
});

