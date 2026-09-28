function loader(content) {
	this.callback(null, content + ' + "queryloader"');
}

export default loader;
