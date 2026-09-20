function loader(content) {
	expect(this.hot).toBe(true);
	return content;
}

export default loader;
