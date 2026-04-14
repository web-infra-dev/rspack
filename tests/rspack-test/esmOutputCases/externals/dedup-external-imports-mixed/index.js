import { ns } from './esm-consumer'
import { cjsResult } from './cjs-bridge'

it('should not have duplicate import identifiers from mixed scope-hoisting', () => {
	expect(ns).toBeDefined()
	expect(cjsResult).toBeDefined()
})
