module.exports = {
	snapshotFileFilter(file) {
		return file === "main.mjs";
	},
	snapshotContent(content) {
		return content
			.split("\n")
			.filter(line => line.includes("rspack_hmr_s_module"))
			.join("\n");
	}
};
