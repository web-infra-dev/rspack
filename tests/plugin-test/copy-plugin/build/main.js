(() => { "use strict"; var r = {}, t = {}; function e(i) { var o = t[i]; if (void 0 !== o) return o.exports; var n = t[i] = { exports: {} }; return r[i](n, n.exports, e), n.exports } e.g = function () { if ("object" == typeof globalThis) return globalThis; try { return this || Function("return this")() } catch (r) { if ("object" == typeof window) return window } }(), e.rv = function () { return "1.0.0-alpha.4" }, (() => { e.g.importScripts && (r = e.g.location + ""); var r, t = e.g.document; if (!r && t && (t.currentScript && (r = t.currentScript.src), !r)) { var i = t.getElementsByTagName("script"); if (i.length) { for (var o = i.length - 1; o > -1 && (!r || !/^http(s?):/.test(r));)r = i[o--].src } } if (!r) throw Error("Automatic publicPath is not supported in this browser"); r = r.replace(/#.*$/, "").replace(/\?.*$/, "").replace(/\/[^\/]+$/, "/"), e.p = r })(), e.ruid = "bundler=rspack@1.0.0-alpha.4", e.p })();