function loader(content) {
	this.callback(null, content + ' + "fragmentloader"');
}

export default loader;
