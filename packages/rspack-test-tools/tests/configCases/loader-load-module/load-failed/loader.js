module.exports = function (code) {
	const callback = this.async();
	this.loadModule(
		`${this.resourcePath}.webpack[javascript/auto]!=!!!./a.js`,
		function (err) {
			expect(err.message).toMatch("Module not found");
			callback(code);
		}
	);
};
