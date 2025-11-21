function loader(content) {
	this.callback(null, content + ' + "fragmentloader"');
}

module.exports = loader;
