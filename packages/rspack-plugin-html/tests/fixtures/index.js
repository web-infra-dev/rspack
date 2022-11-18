"use strict";

require("./common");

require.ensure([], function () {
	require("./async");
});

document.body.innerHTML = document.body.innerHTML + "<p>index.js</p>";
