export default async function (content) {
	if (!content.includes("2")) {
		await this.importModule("./loader2.mjs!./a.js");
	}
	return content;
};
