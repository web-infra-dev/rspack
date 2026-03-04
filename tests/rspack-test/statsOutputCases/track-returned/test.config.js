module.exports = {
	validate(stats) {
		const s = stats.stats ? stats.stats[0] : stats;
		expect(s.compilation.modules.size).toBe(8);
	}
};
