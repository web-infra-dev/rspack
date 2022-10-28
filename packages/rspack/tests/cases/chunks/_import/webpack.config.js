const externalModule = ["uvu", "path", "fs", "expect", "it"];
const external = Object.fromEntries(externalModule.map(x => [x, x]));
module.exports = {
	target: "node",
	externals: external
};
