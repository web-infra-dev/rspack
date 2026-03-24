import * as fsNs from 'fs'

it('should keep namespace import for external namespace member access', () => {
	expect(fsNs.readFile).toBeDefined()
})
