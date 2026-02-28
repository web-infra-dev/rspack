module.exports = async function (content) {
	let res = await this.importModule("./import_module_root.js");
	return content.replace("1", res);
};