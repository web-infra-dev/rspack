import {value} from './module'

it('should have correct export from module', async () => {
	expect(value()).toBe(42)
})
