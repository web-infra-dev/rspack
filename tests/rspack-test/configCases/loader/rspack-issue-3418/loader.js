function loader(content) {
	content += `;module.exports += "-foo"`;
	this.callback(null, content, "");
}

module.exports = loader;
