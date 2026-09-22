module.exports = {
	findBundle(index, options) {
		return Object.keys(options.entry).map(name => `${name}-${index}.js`);
	}
};
