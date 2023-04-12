const request = 'a'
import('./a.js').then(({ a }) => console.log("Literal", a))
import(`./b.js`).then(({b}) => console.log("Template Literal", b))
import(`./${request}.js`).then(({a}) => console.log("context_module_tpl", a))
import('./' + request + '.js').then(({a}) => console.log("context_module_bin", a))
import("./".concat(request, ".js")).then(({a}) => console.log("context_module_concat", a))