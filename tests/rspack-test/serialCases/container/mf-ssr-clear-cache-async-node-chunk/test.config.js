module.exports = {
	findBundle: i => `./main.${i < 2 ? "webpack" : "rspack"}.${i % 2 === 0 ? "async-node" : "node"}.js`
};
