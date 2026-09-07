module.exports = import(/* webpackChunkName: "shared" */ "./shared").then(
	({ value }) => value
);
---
module.exports = "removed";
