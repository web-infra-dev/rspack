module.exports = async function (content) {
	const res = await this.importModule("./import_module_root.js");
	return content.replace("1", res);
};