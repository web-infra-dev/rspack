module.exports = {
	checkStats(_stepName, stats) {
		// should keep module preOrderIndex and postOrderIndex during build
		return [...stats.modules].filter(m => {
			return m.moduleType !== 'runtime'
		}).every((m) => {
			return m.preOrderIndex !== undefined && m.postOrderIndex !== undefined
		})
	}
}
