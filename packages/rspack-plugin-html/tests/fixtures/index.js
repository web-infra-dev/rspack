"use strict";

require("./common");

(async () => await import("./async"))();

document.body.innerHTML = document.body.innerHTML + "<p>index.js</p>";
