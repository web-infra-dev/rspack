import { value } from './foo'

export { value as v }

it('should have deconflicted symbol', () => {
	const path = __non_webpack_require__('path')
	const fs = __non_webpack_require__('fs')

	const code = fs.readFileSync(path.resolve(import.meta.dirname, './main.mjs'), 'utf-8')
	expect(code).toMatchSnapshot(import.meta.dirname)
})
