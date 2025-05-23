module.exports = {
	findBundle: function (i, options) {
		return i === 0 ? "./other.js" : "./module/main.mjs";
	}
};
