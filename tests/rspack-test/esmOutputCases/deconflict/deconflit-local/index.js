import { value } from './foo'

const local = 'index'
const index_local = ''

it('should have deconflicted symbol', () => {
	expect(value).toBe(42)
	expect(local).toBe('index')
	expect(index_local).toBe('')
})
