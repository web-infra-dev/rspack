let builds = 0;

module.exports = function () {
	this.cacheable(false);
	return `export default ${++builds}`;
};
