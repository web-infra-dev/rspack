module.exports.value = 1;
module.exports.getModule = () => module;
module.hot.accept();
---
module.exports.value = 2;
module.exports.getModule = () => module;
module.hot.accept();
