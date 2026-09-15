module.exports = function () {
	const { buildInfo } = this._module;
	buildInfo.trail = [...buildInfo.trail, "worker"];
	buildInfo.parallel = this.parallel;
	this.addBuildDependency("./build.txt");
	return `module.exports = ${this.parallel}`;
};
