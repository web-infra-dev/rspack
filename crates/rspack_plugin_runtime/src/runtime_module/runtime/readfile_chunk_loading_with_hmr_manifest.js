__webpack_require__.hmrM = function () {
	return new Promise(function (resolve, reject) {
		var filename = require('path').join(__dirname, "$OUTPUT_DIR$" + __webpack_require__.hmrF());
		require('fs').readFile(filename, 'utf-8', function (err, content) {
			if (err) {
				if (err.code === "ENOENT") return resolve();
				return reject(err);
			}
			try { resolve(JSON.parse(content)); }
			catch (e) { reject(e); }
		});
	});
}