module.exports = require('./foo')

it('should have correct output for entry re-exports', () => {
	const fs = __non_webpack_require__('fs')
	const path = __non_webpack_require__('path')

	const code = fs.readFileSync(path.resolve(__dirname, './main.mjs'), 'utf-8')

	expect(code).toMatchSnapshot()
})
exports.value = 42
