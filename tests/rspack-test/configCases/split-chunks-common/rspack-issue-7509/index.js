it("should not split chunk when maxInitialRequests set to only 1", function () {
	const a = require("./node_modules/a");
	expect(a).toBe("a");
	const b = require("./node_modules/b");
	expect(b).toBe("b");
	const c = require("./node_modules/c");
	expect(c).toBe("c");
	const d = require("./node_modules/d");
	expect(d).toBe("d");

	const fs = require('fs')
	const files = fs.readdirSync(__dirname)
	expect(files.filter(file => file.endsWith('.js')).length).toBe(1)
});
