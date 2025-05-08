import { value } from './foo'

const local = 'index'
const index_local = ''

it('should have deconflicted symbol', async () => {
	expect(value).toBe(42)
	expect(local).toBe('index')
	expect(index_local).toBe('')
	const {value: v} = await import('./other')
	expect(v).toBe(value)
})
