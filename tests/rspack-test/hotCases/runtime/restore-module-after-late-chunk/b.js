exports.read = () => "not restored";
module.hot.accept();
---
exports.read = () => "not restored";
module.hot.accept();
---
exports.read = () => require("./x");
module.hot.accept();
