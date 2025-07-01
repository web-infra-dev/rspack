// CommonJS module with various export patterns (browser-compatible)
// Simulated path and fs modules for browser environment
const path = {
	normalize: p => p.replace(/[\/\\]+/g, "/").replace(/\/+$/, "") || "/",
	join: (...paths) =>
		paths
			.filter(Boolean)
			.join("/")
			.replace(/[\/\\]+/g, "/"),
	dirname: p => p.replace(/[\/\\][^\/\\]*$/, "") || "/",
	basename: p => p.split(/[\/\\]/).pop() || "",
	extname: p => {
		const m = p.match(/\.[^.\/\\]*$/);
		return m ? m[0] : "";
	},
	resolve: (...paths) =>
		`/${paths
			.filter(Boolean)
			.join("/")
			.replace(/[\/\\]+/g, "/")}`,
	isAbsolute: p => p.startsWith("/"),
	relative: (from, to) => to // Simplified for browser
};

const fs = {
	readFileSync: (path, encoding) => {
		// Simulated file reading for browser
		return `Simulated content of ${path}`;
	},
	existsSync: path => {
		// Simulated file existence check
		return true;
	}
};

// Named exports using exports object
exports.formatPath = function (filePath) {
	return path.normalize(filePath);
};

exports.readFileSync = function (filePath) {
	try {
		return fs.readFileSync(filePath, "utf8");
	} catch (error) {
		return `Error reading file: ${error.message}`;
	}
};

// Object assignment to exports
exports.constants = {
	DEFAULT_ENCODING: "utf8",
	MAX_FILE_SIZE: 1024 * 1024,
	SUPPORTED_FORMATS: ["txt", "json", "js"]
};

// Function assignment to exports
exports.validateFile = function (filePath) {
	const ext = path.extname(filePath).slice(1);
	return this.constants.SUPPORTED_FORMATS.includes(ext);
};

// Class export
class FileManager {
	constructor(basePath = ".") {
		this.basePath = basePath;
	}

	resolve(filePath) {
		return path.resolve(this.basePath, filePath);
	}

	exists(filePath) {
		return fs.existsSync(this.resolve(filePath));
	}
}

exports.FileManager = FileManager;

// Default-style export using module.exports
const defaultUtils = {
	name: "legacy-utils",
	version: "1.0.0",
	type: "commonjs",

	// Methods
	join: (...paths) => path.join(...paths),
	dirname: filePath => path.dirname(filePath),
	basename: filePath => path.basename(filePath),

	// Utility functions
	isAbsolute: filePath => path.isAbsolute(filePath),
	relative: (from, to) => path.relative(from, to)
};

// Mixed pattern: both exports.* and module.exports
module.exports = defaultUtils;

// Additional exports after module.exports (CommonJS allows this)
module.exports.formatPath = exports.formatPath;
module.exports.readFileSync = exports.readFileSync;
module.exports.constants = exports.constants;
module.exports.validateFile = exports.validateFile;
module.exports.FileManager = exports.FileManager;

// Circular reference test
module.exports.getSelf = function () {
	return module.exports;
};
