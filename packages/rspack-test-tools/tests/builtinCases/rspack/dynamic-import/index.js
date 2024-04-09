const request = "a";
import("./child/a.js").then(({ a }) => console.log("Literal", a));
import(`./child/b.js`).then(({ b }) => console.log("Template Literal", b));
import(`./child/${request}.js`).then(({ a }) =>
	console.log("context_module_tpl", a)
);
import("./child/" + request + ".js").then(({ a }) =>
	console.log("context_module_bin", a)
);
import("./child/".concat(request, ".js")).then(({ a }) =>
	console.log("context_module_concat", a)
);
