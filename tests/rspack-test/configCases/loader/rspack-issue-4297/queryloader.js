function loader(content) {
	this.callback(null, content + ' + "queryloader"');
}

module.exports = loader;
