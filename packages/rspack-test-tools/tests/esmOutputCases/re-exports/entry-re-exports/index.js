export * from './foo'


it('should have correct output for entry re-exports', () => {
	const fs = __non_webpack_require__('fs')
	const path = __non_webpack_require__('path')

	console.log(path.resolve(import.meta.dirname, './main.mjs'))

	const code = fs.readFileSync(path.resolve(import.meta.dirname, './main.mjs'), 'utf-8')

	expect(code).toMatchSnapshot()
})
