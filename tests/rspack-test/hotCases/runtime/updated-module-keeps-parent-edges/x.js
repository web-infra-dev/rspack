const p = require("./p");

module.hot.dispose(() => {
	p.load();
});

module.exports = "x";
