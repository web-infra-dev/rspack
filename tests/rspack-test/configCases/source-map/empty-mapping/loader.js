module.exports = function (code, map) {
	expect(map.mappings).toBe('')
	return code
}
