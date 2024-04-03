// @ts-nocheck
const fs = require("fs");
const path = require("path");
const rimraf = require("rimraf");

module.exports = function copyDiff(src, dest, initial) {
	fs.mkdirSync(dest, { recursive: true });
	const files = fs.readdirSync(src);
	files.forEach(filename => {
		const srcFile = path.join(src, filename);
		const destFile = path.join(dest, filename);
		const directory = fs.statSync(srcFile).isDirectory();
		if (directory) {
			copyDiff(srcFile, destFile, initial);
		} else {
			var content = fs.readFileSync(srcFile);
			if (/^DELETE\s*$/.test(content.toString("utf-8"))) {
				fs.unlinkSync(destFile);
			} else if (/^DELETE_DIRECTORY\s*$/.test(content.toString("utf-8"))) {
				rimraf.sync(destFile);
			} else {
				fs.writeFileSync(destFile, content);
				if (initial) {
					const longTimeAgo = Date.now() - 1000 * 60 * 60 * 24;
					fs.utimesSync(
						destFile,
						Date.now() - longTimeAgo,
						Date.now() - longTimeAgo
					);
				}
			}
		}
	});
};
