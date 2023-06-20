function loadUpdateChunk(chunkId, updatedModulesList) {
	return new Promise(function(resolve, reject) {
		var filename = require('path').join(__dirname, "" + __webpack_require__.hu(chunkId));
		require('fs').readFile(filename, 'utf-8', function(err, content) {
			if(err) return reject(err);
			var update = {};
			require('vm').runInThisContext('(function(exports, require, __dirname, __filename) {' + content + '\n})', filename)(update, require, require('path').dirname(filename), filename);
			var updatedModules = update.modules;
			var runtime = update.runtime;
			for(var moduleId in updatedModules) {
				if(__webpack_require__.o(updatedModules, moduleId)) {
					currentUpdate[moduleId] = updatedModules[moduleId];
					if(updatedModulesList) updatedModulesList.push(moduleId);
				}
			}
			if(runtime) currentUpdateRuntime.push(runtime);
			resolve();
		});
	});
}