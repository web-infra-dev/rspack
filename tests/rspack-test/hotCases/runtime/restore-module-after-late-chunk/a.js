exports.read = () => require("./x");
module.hot.accept();
---
exports.read = () => "removed";
module.hot.accept();
---
exports.read = () => "removed";
module.hot.accept();
