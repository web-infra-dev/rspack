const path = require("path");

module.exports = (asset, compiler, stats) => {
	const usedFs = compiler.outputFileSystem;
	const outputPath = stats.compilation.outputOptions.path;

	let data = "";
	let targetFile = asset;

	const queryStringIdx = targetFile.indexOf("?");

	if (queryStringIdx >= 0) {
		targetFile = targetFile.slice(0, queryStringIdx);
	}

	try {
		const isArchive = /.gz$/i.test(targetFile);
		data = usedFs.readFileSync(path.join(outputPath, targetFile));

		if (!isArchive) {
			data = data.toString();
		}
	} catch (error) {
		data = error.toString();
	}

	return data;
};
