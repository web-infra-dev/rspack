import { foo, baz } from './src/barrel'

rs.mock(import('./src/foo'), () => {
  return { value: 'mockedFoo' }
})

rs.mock(import('./src/baz'))

it('should mock modules', async () => {
	rs.doMock(import('./src/bar'), () => {
  	return { value: 'mockedBar' }
	})

	const { bar } = await import('./src/barrel')
	expect(foo).toBe('mockedFoo')
	expect(bar).toBe('mockedBar')
	expect(baz).toBe('mockedBaz')
})
