module.exports = function loader(source) {
	const { number } = this.query;
	return source.split(/\/\* break \*\//)[number - 1];
};
