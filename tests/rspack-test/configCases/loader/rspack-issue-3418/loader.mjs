function loader(content) {
	content += `;module.exports += "-foo"`;
	this.callback(null, content, "");
}

export default loader;
