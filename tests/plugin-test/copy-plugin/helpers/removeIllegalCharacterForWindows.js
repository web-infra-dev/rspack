module.exports = string =>
	process.platform !== "win32" ? string : string.replace(/[*?"<>|]/g, "");
