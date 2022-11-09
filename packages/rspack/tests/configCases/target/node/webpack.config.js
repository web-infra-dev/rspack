const externalModule = ["uvu", "path", "fs", "expect", "os"];
const external = Object.fromEntries(externalModule.map((x) => [x, x]));
module.exports = {
	target: "node",
	externals: external
};
