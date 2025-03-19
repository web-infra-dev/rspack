import v from './lib'

function checkStats(stats) {
	// should keep module preOrderIndex and postOrderIndex during build
	return [...stats.modules].filter(m => {
		return m.moduleType !== 'runtime'
	}).every((m) => {
		return m.preOrderIndex !== undefined && m.postOrderIndex !== undefined
	})
}

it('should compile', () => {
	expect(v).toBe(1)
	expect(checkStats(__STATS__)).toBe(true)
})
