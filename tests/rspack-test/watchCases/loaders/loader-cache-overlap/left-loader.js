const runs = new Map();

module.exports = function(source) {
	const count = (runs.get(this.resource) || 0) + 1;
	runs.set(this.resource, count);
	return `${source}\nmodule.exports.leftRuns = ${count};`;
};
