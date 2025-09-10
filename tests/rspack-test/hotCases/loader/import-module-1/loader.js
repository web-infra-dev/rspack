module.exports = async function (content) {
	if (!content.includes("2")) {
		await this.importModule("./loader2.js!./a.js");
	}
	return content;
};
