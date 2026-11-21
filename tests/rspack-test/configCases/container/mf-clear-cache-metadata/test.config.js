module.exports = {
	findBundle: i =>
		i === 0 ? ["./bundle.js", "./no-remote.js"] : i === 1 ? "./plain-node.js" : undefined
};
