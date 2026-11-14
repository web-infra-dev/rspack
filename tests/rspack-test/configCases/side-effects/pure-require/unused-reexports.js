let calls = 0;
function before() {
	calls++;
}

// Missing semicolons are intentional: removing the assignment exposes the require.
before()
exports.unused = require("./declared");
before()
module.exports.unusedMember = require("./declared").value;
before()
this.unused = require("./declared");

exports.used = calls;
