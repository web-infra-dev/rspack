import defer * as ns from './lib-defer.js' // this is a defer module
import { lib } from './lib.js' // this has tla

it('should has correct exec order and export value', async () => {
	const { demo } = await import(/*webpackIgnore: true*/ './main.mjs')
	expect(demo).toBe(84)
})

const demo = lib + ns.v
export { demo }
