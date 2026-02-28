import fs from 'fs'
import path from 'path'

it('should have extra chunk', () => {
	expect(fs.existsSync(path.resolve(__dirname, 'foo-index_js.js'))).toBe(true)
})
