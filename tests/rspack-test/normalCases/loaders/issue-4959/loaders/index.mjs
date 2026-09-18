export default function() {
	var callback = this.async();
	this.loadModule("b", function(error) {
		if (error) {
			return callback(error);
		}
		callback(null, "module.exports = require('b');");
	});
};
