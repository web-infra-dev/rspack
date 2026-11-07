require("./state").push("initial");
module.hot.accept();
---
require("./state").push("self");
module.hot.accept();
