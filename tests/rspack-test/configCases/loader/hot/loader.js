function loader(content) {
	expect(this.hot).toBe(true);
	return content;
}

module.exports = loader;
