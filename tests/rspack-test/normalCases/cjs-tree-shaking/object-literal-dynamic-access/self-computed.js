const dynamic = name => name;
module.exports = {
	used: "used",
	unused: "unused",
	read: name => module.exports[dynamic(name)],
	usedExports: __webpack_exports_info__.usedExports
};
