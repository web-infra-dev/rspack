(function() {// runtime instance
var runtime = new Object();
self["__rspack_runtime__"] = runtime;
// mount Modules
(function () {
	runtime.installedModules = {
"./index.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _testJs = __rspack_runtime__.interopRequire(__rspack_require__("./test.js"));
_testJs.default;
},
"./test.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
!function(t, e) {
    "object" == typeof exports && "object" == typeof module ? module.exports = e() : "function" == typeof define && define.amd ? define([], e) : "object" == typeof exports ? exports.Pickr = e() : t.Pickr = e();
}(self, function() {
    return (()=>{
        "use strict";
        var t = {
            d: (e, o)=>{
                for(var n in o)t.o(o, n) && !t.o(e, n) && Object.defineProperty(e, n, {
                    enumerable: !0,
                    get: o[n]
                });
            },
            o: (t, e)=>Object.prototype.hasOwnProperty.call(t, e),
            r: (t)=>{
                "undefined" != typeof Symbol && Symbol.toStringTag && Object.defineProperty(t, Symbol.toStringTag, {
                    value: "Module"
                }), Object.defineProperty(t, "__esModule", {
                    value: !0
                });
            }
        }, e = {};
        t.d(e, {
            default: ()=>L
        });
        var o = {};
        function n(t, e, o, n, i = {}) {
            e instanceof HTMLCollection || e instanceof NodeList ? e = Array.from(e) : Array.isArray(e) || (e = [
                e
            ]), Array.isArray(o) || (o = [
                o
            ]);
            for (const s of e)for (const e1 of o)s[t](e1, n, {
                capture: !1,
                ...i
            });
            return Array.prototype.slice.call(arguments, 1);
        }
        t.r(o), t.d(o, {
            adjustableInputNumbers: ()=>p,
            createElementFromString: ()=>r,
            createFromTemplate: ()=>a,
            eventPath: ()=>l,
            off: ()=>s,
            on: ()=>i,
            resolveElement: ()=>c
        });
        const i = n.bind(null, "addEventListener"), s = n.bind(null, "removeEventListener");
        function r(t) {
            const e = document.createElement("div");
            return e.innerHTML = t.trim(), e.firstElementChild;
        }
        function a(t) {
            const e = (t, e)=>{
                const o = t.getAttribute(e);
                return t.removeAttribute(e), o;
            }, o = (t, n = {})=>{
                const i = e(t, ":obj"), s = e(t, ":ref"), r = i ? n[i] = {} : n;
                s && (n[s] = t);
                for (const n1 of Array.from(t.children)){
                    const t1 = e(n1, ":arr"), i1 = o(n1, t1 ? {} : r);
                    t1 && (r[t1] || (r[t1] = [])).push(Object.keys(i1).length ? i1 : n1);
                }
                return n;
            };
            return o(r(t));
        }
        function l(t) {
            let e = t.path || t.composedPath && t.composedPath();
            if (e) return e;
            let o = t.target.parentElement;
            for(e = [
                t.target,
                o
            ]; o = o.parentElement;)e.push(o);
            return e.push(document, window), e;
        }
        function c(t) {
            return t instanceof Element ? t : "string" == typeof t ? t.split(/>>/g).reduce((t, e, o, n)=>(t = t.querySelector(e), o < n.length - 1 ? t.shadowRoot : t), document) : null;
        }
        function p(t, e = (t)=>t) {
            function o(o) {
                const n = [
                    .001,
                    .01,
                    .1
                ][Number(o.shiftKey || 2 * o.ctrlKey)] * (o.deltaY < 0 ? 1 : -1);
                let i = 0, s = t.selectionStart;
                t.value = t.value.replace(/[\d.]+/g, (t, o)=>o <= s && o + t.length >= s ? (s = o, e(Number(t), n, i)) : (i++, t)), t.focus(), t.setSelectionRange(s, s), o.preventDefault(), t.dispatchEvent(new Event("input"));
            }
            i(t, "focus", ()=>i(window, "wheel", o, {
                    passive: !1
                })), i(t, "blur", ()=>s(window, "wheel", o));
        }
        const { min: u , max: h , floor: d , round: m  } = Math;
        function f(t, e, o) {
            e /= 100, o /= 100;
            const n = d(t = t / 360 * 6), i = t - n, s = o * (1 - e), r = o * (1 - i * e), a = o * (1 - (1 - i) * e), l = n % 6;
            return [
                255 * [
                    o,
                    r,
                    s,
                    s,
                    a,
                    o
                ][l],
                255 * [
                    a,
                    o,
                    o,
                    r,
                    s,
                    s
                ][l],
                255 * [
                    s,
                    s,
                    a,
                    o,
                    o,
                    r
                ][l]
            ];
        }
        function v(t, e, o) {
            const n = (2 - (e /= 100)) * (o /= 100) / 2;
            return 0 !== n && (e = 1 === n ? 0 : n < .5 ? e * o / (2 * n) : e * o / (2 - 2 * n)), [
                t,
                100 * e,
                100 * n
            ];
        }
        function b(t, e, o) {
            const n = u(t /= 255, e /= 255, o /= 255), i = h(t, e, o), s = i - n;
            let r, a;
            if (0 === s) r = a = 0;
            else {
                a = s / i;
                const n1 = ((i - t) / 6 + s / 2) / s, l = ((i - e) / 6 + s / 2) / s, c = ((i - o) / 6 + s / 2) / s;
                t === i ? r = c - l : e === i ? r = 1 / 3 + n1 - c : o === i && (r = 2 / 3 + l - n1), r < 0 ? r += 1 : r > 1 && (r -= 1);
            }
            return [
                360 * r,
                100 * a,
                100 * i
            ];
        }
        function y(t, e, o, n) {
            e /= 100, o /= 100;
            return [
                ...b(255 * (1 - u(1, (t /= 100) * (1 - (n /= 100)) + n)), 255 * (1 - u(1, e * (1 - n) + n)), 255 * (1 - u(1, o * (1 - n) + n)))
            ];
        }
        function g(t, e, o) {
            e /= 100;
            const n = 2 * (e *= (o /= 100) < .5 ? o : 1 - o) / (o + e) * 100, i = 100 * (o + e);
            return [
                t,
                isNaN(n) ? 0 : n,
                i
            ];
        }
        function _(t) {
            return b(...t.match(/.{2}/g).map((t)=>parseInt(t, 16)));
        }
        function w(t) {
            t = t.match(/^[a-zA-Z]+$/) ? function(t) {
                if ("black" === t.toLowerCase()) return "#000";
                const e = document.createElement("canvas").getContext("2d");
                return e.fillStyle = t, "#000" === e.fillStyle ? null : e.fillStyle;
            }(t) : t;
            const e = {
                cmyk: /^cmyk[\D]+([\d.]+)[\D]+([\d.]+)[\D]+([\d.]+)[\D]+([\d.]+)/i,
                rgba: /^((rgba)|rgb)[\D]+([\d.]+)[\D]+([\d.]+)[\D]+([\d.]+)[\D]*?([\d.]+|$)/i,
                hsla: /^((hsla)|hsl)[\D]+([\d.]+)[\D]+([\d.]+)[\D]+([\d.]+)[\D]*?([\d.]+|$)/i,
                hsva: /^((hsva)|hsv)[\D]+([\d.]+)[\D]+([\d.]+)[\D]+([\d.]+)[\D]*?([\d.]+|$)/i,
                hexa: /^#?(([\dA-Fa-f]{3,4})|([\dA-Fa-f]{6})|([\dA-Fa-f]{8}))$/i
            }, o = (t)=>t.map((t)=>/^(|\d+)\.\d+|\d+$/.test(t) ? Number(t) : void 0);
            let n;
            t: for(const i in e){
                if (!(n = e[i].exec(t))) continue;
                const s = (t)=>!!n[2] == ("number" == typeof t);
                switch(i){
                    case "cmyk":
                        {
                            const [, t1, e1, s1, r] = o(n);
                            if (t1 > 100 || e1 > 100 || s1 > 100 || r > 100) break t;
                            return {
                                values: y(t1, e1, s1, r),
                                type: i
                            };
                        }
                    case "rgba":
                        {
                            const [, , , t2, e2, r1, a] = o(n);
                            if (t2 > 255 || e2 > 255 || r1 > 255 || a < 0 || a > 1 || !s(a)) break t;
                            return {
                                values: [
                                    ...b(t2, e2, r1),
                                    a
                                ],
                                a,
                                type: i
                            };
                        }
                    case "hexa":
                        {
                            let [, t3] = n;
                            4 !== t3.length && 3 !== t3.length || (t3 = t3.split("").map((t)=>t + t).join(""));
                            const e3 = t3.substring(0, 6);
                            let o1 = t3.substring(6);
                            return o1 = o1 ? parseInt(o1, 16) / 255 : void 0, {
                                values: [
                                    ..._(e3),
                                    o1
                                ],
                                a: o1,
                                type: i
                            };
                        }
                    case "hsla":
                        {
                            const [, , , t4, e4, r2, a1] = o(n);
                            if (t4 > 360 || e4 > 100 || r2 > 100 || a1 < 0 || a1 > 1 || !s(a1)) break t;
                            return {
                                values: [
                                    ...g(t4, e4, r2),
                                    a1
                                ],
                                a: a1,
                                type: i
                            };
                        }
                    case "hsva":
                        {
                            const [, , , t5, e5, r3, a2] = o(n);
                            if (t5 > 360 || e5 > 100 || r3 > 100 || a2 < 0 || a2 > 1 || !s(a2)) break t;
                            return {
                                values: [
                                    t5,
                                    e5,
                                    r3,
                                    a2
                                ],
                                a: a2,
                                type: i
                            };
                        }
                }
            }
            return {
                values: null,
                type: null
            };
        }
        function A(t = 0, e = 0, o = 0, n = 1) {
            const i = (t, e)=>(o = -1)=>e(~o ? t.map((t)=>Number(t.toFixed(o))) : t), s = {
                h: t,
                s: e,
                v: o,
                a: n,
                toHSVA () {
                    const t = [
                        s.h,
                        s.s,
                        s.v,
                        s.a
                    ];
                    return t.toString = i(t, (t)=>`hsva(${t[0]}, ${t[1]}%, ${t[2]}%, ${s.a})`), t;
                },
                toHSLA () {
                    const t = [
                        ...v(s.h, s.s, s.v),
                        s.a
                    ];
                    return t.toString = i(t, (t)=>`hsla(${t[0]}, ${t[1]}%, ${t[2]}%, ${s.a})`), t;
                },
                toRGBA () {
                    const t = [
                        ...f(s.h, s.s, s.v),
                        s.a
                    ];
                    return t.toString = i(t, (t)=>`rgba(${t[0]}, ${t[1]}, ${t[2]}, ${s.a})`), t;
                },
                toCMYK () {
                    const t = function(t, e, o) {
                        const n = f(t, e, o), i = n[0] / 255, s = n[1] / 255, r = n[2] / 255, a = u(1 - i, 1 - s, 1 - r);
                        return [
                            100 * (1 === a ? 0 : (1 - i - a) / (1 - a)),
                            100 * (1 === a ? 0 : (1 - s - a) / (1 - a)),
                            100 * (1 === a ? 0 : (1 - r - a) / (1 - a)),
                            100 * a
                        ];
                    }(s.h, s.s, s.v);
                    return t.toString = i(t, (t)=>`cmyk(${t[0]}%, ${t[1]}%, ${t[2]}%, ${t[3]}%)`), t;
                },
                toHEXA () {
                    const t = function(t, e, o) {
                        return f(t, e, o).map((t)=>m(t).toString(16).padStart(2, "0"));
                    }(s.h, s.s, s.v), e = s.a >= 1 ? "" : Number((255 * s.a).toFixed(0)).toString(16).toUpperCase().padStart(2, "0");
                    return e && t.push(e), t.toString = ()=>`#${t.join("").toUpperCase()}`, t;
                },
                clone: ()=>A(s.h, s.s, s.v, s.a)
            };
            return s;
        }
        const C = (t)=>Math.max(Math.min(t, 1), 0);
        function $(t) {
            const e = {
                options: Object.assign({
                    lock: null,
                    onchange: ()=>0,
                    onstop: ()=>0
                }, t),
                _keyboard (t) {
                    const { options: o  } = e, { type: n , key: i  } = t;
                    if (document.activeElement === o.wrapper) {
                        const { lock: o1  } = e.options, s = "ArrowUp" === i, r = "ArrowRight" === i, a = "ArrowDown" === i, l = "ArrowLeft" === i;
                        if ("keydown" === n && (s || r || a || l)) {
                            let n1 = 0, i1 = 0;
                            "v" === o1 ? n1 = s || r ? 1 : -1 : "h" === o1 ? n1 = s || r ? -1 : 1 : (i1 = s ? -1 : a ? 1 : 0, n1 = l ? -1 : r ? 1 : 0), e.update(C(e.cache.x + .01 * n1), C(e.cache.y + .01 * i1)), t.preventDefault();
                        } else i.startsWith("Arrow") && (e.options.onstop(), t.preventDefault());
                    }
                },
                _tapstart (t) {
                    i(document, [
                        "mouseup",
                        "touchend",
                        "touchcancel"
                    ], e._tapstop), i(document, [
                        "mousemove",
                        "touchmove"
                    ], e._tapmove), t.cancelable && t.preventDefault(), e._tapmove(t);
                },
                _tapmove (t) {
                    const { options: o , cache: n  } = e, { lock: i , element: s , wrapper: r  } = o, a = r.getBoundingClientRect();
                    let l = 0, c = 0;
                    if (t) {
                        const e1 = t && t.touches && t.touches[0];
                        l = t ? (e1 || t).clientX : 0, c = t ? (e1 || t).clientY : 0, l < a.left ? l = a.left : l > a.left + a.width && (l = a.left + a.width), c < a.top ? c = a.top : c > a.top + a.height && (c = a.top + a.height), l -= a.left, c -= a.top;
                    } else n && (l = n.x * a.width, c = n.y * a.height);
                    "h" !== i && (s.style.left = `calc(${l / a.width * 100}% - ${s.offsetWidth / 2}px)`), "v" !== i && (s.style.top = `calc(${c / a.height * 100}% - ${s.offsetHeight / 2}px)`), e.cache = {
                        x: l / a.width,
                        y: c / a.height
                    };
                    const p = C(l / a.width), u = C(c / a.height);
                    switch(i){
                        case "v":
                            return o.onchange(p);
                        case "h":
                            return o.onchange(u);
                        default:
                            return o.onchange(p, u);
                    }
                },
                _tapstop () {
                    e.options.onstop(), s(document, [
                        "mouseup",
                        "touchend",
                        "touchcancel"
                    ], e._tapstop), s(document, [
                        "mousemove",
                        "touchmove"
                    ], e._tapmove);
                },
                trigger () {
                    e._tapmove();
                },
                update (t = 0, o = 0) {
                    const { left: n , top: i , width: s , height: r  } = e.options.wrapper.getBoundingClientRect();
                    "h" === e.options.lock && (o = t), e._tapmove({
                        clientX: n + s * t,
                        clientY: i + r * o
                    });
                },
                destroy () {
                    const { options: t , _tapstart: o , _keyboard: n  } = e;
                    s(document, [
                        "keydown",
                        "keyup"
                    ], n), s([
                        t.wrapper,
                        t.element
                    ], "mousedown", o), s([
                        t.wrapper,
                        t.element
                    ], "touchstart", o, {
                        passive: !1
                    });
                }
            }, { options: o , _tapstart: n , _keyboard: r  } = e;
            return i([
                o.wrapper,
                o.element
            ], "mousedown", n), i([
                o.wrapper,
                o.element
            ], "touchstart", n, {
                passive: !1
            }), i(document, [
                "keydown",
                "keyup"
            ], r), e;
        }
        function k(t = {}) {
            t = Object.assign({
                onchange: ()=>0,
                className: "",
                elements: []
            }, t);
            const e = i(t.elements, "click", (e)=>{
                t.elements.forEach((o)=>o.classList[e.target === o ? "add" : "remove"](t.className)), t.onchange(e), e.stopPropagation();
            });
            return {
                destroy: ()=>s(...e)
            };
        }
        const S = {
            variantFlipOrder: {
                start: "sme",
                middle: "mse",
                end: "ems"
            },
            positionFlipOrder: {
                top: "tbrl",
                right: "rltb",
                bottom: "btrl",
                left: "lrbt"
            },
            position: "bottom",
            margin: 8
        }, O = (t, e, o)=>{
            const { container: n , margin: i , position: s , variantFlipOrder: r , positionFlipOrder: a  } = {
                container: document.documentElement.getBoundingClientRect(),
                ...S,
                ...o
            }, { left: l , top: c  } = e.style;
            e.style.left = "0", e.style.top = "0";
            const p = t.getBoundingClientRect(), u = e.getBoundingClientRect(), h = {
                t: p.top - u.height - i,
                b: p.bottom + i,
                r: p.right + i,
                l: p.left - u.width - i
            }, d = {
                vs: p.left,
                vm: p.left + p.width / 2 + -u.width / 2,
                ve: p.left + p.width - u.width,
                hs: p.top,
                hm: p.bottom - p.height / 2 - u.height / 2,
                he: p.bottom - u.height
            }, [m, f = "middle"] = s.split("-"), v = a[m], b = r[f], { top: y , left: g , bottom: _ , right: w  } = n;
            for (const t1 of v){
                const o1 = "t" === t1 || "b" === t1, n1 = h[t1], [i1, s1] = o1 ? [
                    "top",
                    "left"
                ] : [
                    "left",
                    "top"
                ], [r1, a1] = o1 ? [
                    u.height,
                    u.width
                ] : [
                    u.width,
                    u.height
                ], [l1, c1] = o1 ? [
                    _,
                    w
                ] : [
                    w,
                    _
                ], [p1, m1] = o1 ? [
                    y,
                    g
                ] : [
                    g,
                    y
                ];
                if (!(n1 < p1 || n1 + r1 > l1)) for (const r2 of b){
                    const l2 = d[(o1 ? "v" : "h") + r2];
                    if (!(l2 < m1 || l2 + a1 > c1)) return e.style[s1] = l2 - u[s1] + "px", e.style[i1] = n1 - u[i1] + "px", t1 + r2;
                }
            }
            return e.style.left = l, e.style.top = c, null;
        };
        function E(t, e, o) {
            return e in t ? Object.defineProperty(t, e, {
                value: o,
                enumerable: !0,
                configurable: !0,
                writable: !0
            }) : t[e] = o, t;
        }
        class L {
            constructor(t){
                E(this, "_initializingActive", !0), E(this, "_recalc", !0), E(this, "_nanopop", null), E(this, "_root", null), E(this, "_color", A()), E(this, "_lastColor", A()), E(this, "_swatchColors", []), E(this, "_setupAnimationFrame", null), E(this, "_eventListener", {
                    init: [],
                    save: [],
                    hide: [],
                    show: [],
                    clear: [],
                    change: [],
                    changestop: [],
                    cancel: [],
                    swatchselect: []
                }), this.options = t = Object.assign({
                    ...L.DEFAULT_OPTIONS
                }, t);
                const { swatches: e , components: o , theme: n , sliders: i , lockOpacity: s , padding: r  } = t;
                [
                    "nano",
                    "monolith"
                ].includes(n) && !i && (t.sliders = "h"), o.interaction || (o.interaction = {});
                const { preview: a , opacity: l , hue: c , palette: p  } = o;
                o.opacity = !s && l, o.palette = p || a || l || c, this._preBuild(), this._buildComponents(), this._bindEvents(), this._finalBuild(), e && e.length && e.forEach((t)=>this.addSwatch(t));
                const { button: u , app: h  } = this._root;
                this._nanopop = ((t, e, o)=>{
                    const n = "object" != typeof t || t instanceof HTMLElement ? {
                        reference: t,
                        popper: e,
                        ...o
                    } : t;
                    return {
                        update (t = n) {
                            const { reference: e , popper: o  } = Object.assign(n, t);
                            if (!o || !e) throw new Error("Popper- or reference-element missing.");
                            return O(e, o, n);
                        }
                    };
                })(u, h, {
                    margin: r
                }), u.setAttribute("role", "button"), u.setAttribute("aria-label", this._t("btn:toggle"));
                const d = this;
                this._setupAnimationFrame = requestAnimationFrame(function e() {
                    if (!h.offsetWidth) return requestAnimationFrame(e);
                    d.setColor(t.default), d._rePositioningPicker(), t.defaultRepresentation && (d._representation = t.defaultRepresentation, d.setColorRepresentation(d._representation)), t.showAlways && d.show(), d._initializingActive = !1, d._emit("init");
                });
            }
            _preBuild() {
                const { options: t  } = this;
                for (const e of [
                    "el",
                    "container"
                ])t[e] = c(t[e]);
                this._root = ((t)=>{
                    const { components: e , useAsButton: o , inline: n , appClass: i , theme: s , lockOpacity: r  } = t.options, l = (t)=>t ? "" : 'style="display:none" hidden', c = (e)=>t._t(e), p = a(`\n      <div :ref="root" class="pickr">\n\n        ${o ? "" : '<button type="button" :ref="button" class="pcr-button"></button>'}\n\n        <div :ref="app" class="pcr-app ${i || ""}" data-theme="${s}" ${n ? 'style="position: unset"' : ""} aria-label="${c("ui:dialog")}" role="window">\n          <div class="pcr-selection" ${l(e.palette)}>\n            <div :obj="preview" class="pcr-color-preview" ${l(e.preview)}>\n              <button type="button" :ref="lastColor" class="pcr-last-color" aria-label="${c("btn:last-color")}"></button>\n              <div :ref="currentColor" class="pcr-current-color"></div>\n            </div>\n\n            <div :obj="palette" class="pcr-color-palette">\n              <div :ref="picker" class="pcr-picker"></div>\n              <div :ref="palette" class="pcr-palette" tabindex="0" aria-label="${c("aria:palette")}" role="listbox"></div>\n            </div>\n\n            <div :obj="hue" class="pcr-color-chooser" ${l(e.hue)}>\n              <div :ref="picker" class="pcr-picker"></div>\n              <div :ref="slider" class="pcr-hue pcr-slider" tabindex="0" aria-label="${c("aria:hue")}" role="slider"></div>\n            </div>\n\n            <div :obj="opacity" class="pcr-color-opacity" ${l(e.opacity)}>\n              <div :ref="picker" class="pcr-picker"></div>\n              <div :ref="slider" class="pcr-opacity pcr-slider" tabindex="0" aria-label="${c("aria:opacity")}" role="slider"></div>\n            </div>\n          </div>\n\n          <div class="pcr-swatches ${e.palette ? "" : "pcr-last"}" :ref="swatches"></div>\n\n          <div :obj="interaction" class="pcr-interaction" ${l(Object.keys(e.interaction).length)}>\n            <input :ref="result" class="pcr-result" type="text" spellcheck="false" ${l(e.interaction.input)} aria-label="${c("aria:input")}">\n\n            <input :arr="options" class="pcr-type" data-type="HEXA" value="${r ? "HEX" : "HEXA"}" type="button" ${l(e.interaction.hex)}>\n            <input :arr="options" class="pcr-type" data-type="RGBA" value="${r ? "RGB" : "RGBA"}" type="button" ${l(e.interaction.rgba)}>\n            <input :arr="options" class="pcr-type" data-type="HSLA" value="${r ? "HSL" : "HSLA"}" type="button" ${l(e.interaction.hsla)}>\n            <input :arr="options" class="pcr-type" data-type="HSVA" value="${r ? "HSV" : "HSVA"}" type="button" ${l(e.interaction.hsva)}>\n            <input :arr="options" class="pcr-type" data-type="CMYK" value="CMYK" type="button" ${l(e.interaction.cmyk)}>\n\n            <input :ref="save" class="pcr-save" value="${c("btn:save")}" type="button" ${l(e.interaction.save)} aria-label="${c("aria:btn:save")}">\n            <input :ref="cancel" class="pcr-cancel" value="${c("btn:cancel")}" type="button" ${l(e.interaction.cancel)} aria-label="${c("aria:btn:cancel")}">\n            <input :ref="clear" class="pcr-clear" value="${c("btn:clear")}" type="button" ${l(e.interaction.clear)} aria-label="${c("aria:btn:clear")}">\n          </div>\n        </div>\n      </div>\n    `), u = p.interaction;
                    return u.options.find((t)=>!t.hidden && !t.classList.add("active")), u.type = ()=>u.options.find((t)=>t.classList.contains("active")), p;
                })(this), t.useAsButton && (this._root.button = t.el), t.container.appendChild(this._root.root);
            }
            _finalBuild() {
                const t = this.options, e = this._root;
                if (t.container.removeChild(e.root), t.inline) {
                    const o = t.el.parentElement;
                    t.el.nextSibling ? o.insertBefore(e.app, t.el.nextSibling) : o.appendChild(e.app);
                } else t.container.appendChild(e.app);
                t.useAsButton ? t.inline && t.el.remove() : t.el.parentNode.replaceChild(e.root, t.el), t.disabled && this.disable(), t.comparison || (e.button.style.transition = "none", t.useAsButton || (e.preview.lastColor.style.transition = "none")), this.hide();
            }
            _buildComponents() {
                const t = this, e = this.options.components, o = (t.options.sliders || "v").repeat(2), [n, i] = o.match(/^[vh]+$/g) ? o : [], s = ()=>this._color || (this._color = this._lastColor.clone()), r = {
                    palette: $({
                        element: t._root.palette.picker,
                        wrapper: t._root.palette.palette,
                        onstop: ()=>t._emit("changestop", "slider", t),
                        onchange (o, n) {
                            if (!e.palette) return;
                            const i = s(), { _root: r , options: a  } = t, { lastColor: l , currentColor: c  } = r.preview;
                            t._recalc && (i.s = 100 * o, i.v = 100 - 100 * n, i.v < 0 && (i.v = 0), t._updateOutput("slider"));
                            const p = i.toRGBA().toString(0);
                            this.element.style.background = p, this.wrapper.style.background = `\n                        linear-gradient(to top, rgba(0, 0, 0, ${i.a}), transparent),\n                        linear-gradient(to left, hsla(${i.h}, 100%, 50%, ${i.a}), rgba(255, 255, 255, ${i.a}))\n                    `, a.comparison ? a.useAsButton || t._lastColor || l.style.setProperty("--pcr-color", p) : (r.button.style.setProperty("--pcr-color", p), r.button.classList.remove("clear"));
                            const u = i.toHEXA().toString();
                            for (const { el: e1 , color: o1  } of t._swatchColors)e1.classList[u === o1.toHEXA().toString() ? "add" : "remove"]("pcr-active");
                            c.style.setProperty("--pcr-color", p);
                        }
                    }),
                    hue: $({
                        lock: "v" === i ? "h" : "v",
                        element: t._root.hue.picker,
                        wrapper: t._root.hue.slider,
                        onstop: ()=>t._emit("changestop", "slider", t),
                        onchange (o) {
                            if (!e.hue || !e.palette) return;
                            const n = s();
                            t._recalc && (n.h = 360 * o), this.element.style.backgroundColor = `hsl(${n.h}, 100%, 50%)`, r.palette.trigger();
                        }
                    }),
                    opacity: $({
                        lock: "v" === n ? "h" : "v",
                        element: t._root.opacity.picker,
                        wrapper: t._root.opacity.slider,
                        onstop: ()=>t._emit("changestop", "slider", t),
                        onchange (o) {
                            if (!e.opacity || !e.palette) return;
                            const n = s();
                            t._recalc && (n.a = Math.round(100 * o) / 100), this.element.style.background = `rgba(0, 0, 0, ${n.a})`, r.palette.trigger();
                        }
                    }),
                    selectable: k({
                        elements: t._root.interaction.options,
                        className: "active",
                        onchange (e) {
                            t._representation = e.target.getAttribute("data-type").toUpperCase(), t._recalc && t._updateOutput("swatch");
                        }
                    })
                };
                this._components = r;
            }
            _bindEvents() {
                const { _root: t , options: e  } = this, o = [
                    i(t.interaction.clear, "click", ()=>this._clearColor()),
                    i([
                        t.interaction.cancel,
                        t.preview.lastColor
                    ], "click", ()=>{
                        this.setHSVA(...(this._lastColor || this._color).toHSVA(), !0), this._emit("cancel");
                    }),
                    i(t.interaction.save, "click", ()=>{
                        !this.applyColor() && !e.showAlways && this.hide();
                    }),
                    i(t.interaction.result, [
                        "keyup",
                        "input"
                    ], (t)=>{
                        this.setColor(t.target.value, !0) && !this._initializingActive && (this._emit("change", this._color, "input", this), this._emit("changestop", "input", this)), t.stopImmediatePropagation();
                    }),
                    i(t.interaction.result, [
                        "focus",
                        "blur"
                    ], (t)=>{
                        this._recalc = "blur" === t.type, this._recalc && this._updateOutput(null);
                    }),
                    i([
                        t.palette.palette,
                        t.palette.picker,
                        t.hue.slider,
                        t.hue.picker,
                        t.opacity.slider,
                        t.opacity.picker
                    ], [
                        "mousedown",
                        "touchstart"
                    ], ()=>this._recalc = !0, {
                        passive: !0
                    })
                ];
                if (!e.showAlways) {
                    const n = e.closeWithKey;
                    o.push(i(t.button, "click", ()=>this.isOpen() ? this.hide() : this.show()), i(document, "keyup", (t)=>this.isOpen() && (t.key === n || t.code === n) && this.hide()), i(document, [
                        "touchstart",
                        "mousedown"
                    ], (e)=>{
                        this.isOpen() && !l(e).some((e)=>e === t.app || e === t.button) && this.hide();
                    }, {
                        capture: !0
                    }));
                }
                if (e.adjustableNumbers) {
                    const e1 = {
                        rgba: [
                            255,
                            255,
                            255,
                            1
                        ],
                        hsva: [
                            360,
                            100,
                            100,
                            1
                        ],
                        hsla: [
                            360,
                            100,
                            100,
                            1
                        ],
                        cmyk: [
                            100,
                            100,
                            100,
                            100
                        ]
                    };
                    p(t.interaction.result, (t, o, n)=>{
                        const i = e1[this.getColorRepresentation().toLowerCase()];
                        if (i) {
                            const e = i[n], s = t + (e >= 100 ? 1e3 * o : o);
                            return s <= 0 ? 0 : Number((s < e ? s : e).toPrecision(3));
                        }
                        return t;
                    });
                }
                if (e.autoReposition && !e.inline) {
                    let t1 = null;
                    const n1 = this;
                    o.push(i(window, [
                        "scroll",
                        "resize"
                    ], ()=>{
                        n1.isOpen() && (e.closeOnScroll && n1.hide(), null === t1 ? (t1 = setTimeout(()=>t1 = null, 100), requestAnimationFrame(function e() {
                            n1._rePositioningPicker(), null !== t1 && requestAnimationFrame(e);
                        })) : (clearTimeout(t1), t1 = setTimeout(()=>t1 = null, 100)));
                    }, {
                        capture: !0
                    }));
                }
                this._eventBindings = o;
            }
            _rePositioningPicker() {
                const { options: t  } = this;
                if (!t.inline) {
                    if (!this._nanopop.update({
                        container: document.body.getBoundingClientRect(),
                        position: t.position
                    })) {
                        const t1 = this._root.app, e = t1.getBoundingClientRect();
                        t1.style.top = (window.innerHeight - e.height) / 2 + "px", t1.style.left = (window.innerWidth - e.width) / 2 + "px";
                    }
                }
            }
            _updateOutput(t) {
                const { _root: e , _color: o , options: n  } = this;
                if (e.interaction.type()) {
                    const t1 = `to${e.interaction.type().getAttribute("data-type")}`;
                    e.interaction.result.value = "function" == typeof o[t1] ? o[t1]().toString(n.outputPrecision) : "";
                }
                !this._initializingActive && this._recalc && this._emit("change", o, t, this);
            }
            _clearColor(t = !1) {
                const { _root: e , options: o  } = this;
                o.useAsButton || e.button.style.setProperty("--pcr-color", "rgba(0, 0, 0, 0.15)"), e.button.classList.add("clear"), o.showAlways || this.hide(), this._lastColor = null, this._initializingActive || t || (this._emit("save", null), this._emit("clear"));
            }
            _parseLocalColor(t) {
                const { values: e , type: o , a: n  } = w(t), { lockOpacity: i  } = this.options, s = void 0 !== n && 1 !== n;
                return e && 3 === e.length && (e[3] = void 0), {
                    values: !e || i && s ? null : e,
                    type: o
                };
            }
            _t(t) {
                return this.options.i18n[t] || L.I18N_DEFAULTS[t];
            }
            _emit(t, ...e) {
                this._eventListener[t].forEach((t)=>t(...e, this));
            }
            on(t, e) {
                return this._eventListener[t].push(e), this;
            }
            off(t, e) {
                const o = this._eventListener[t] || [], n = o.indexOf(e);
                return ~n && o.splice(n, 1), this;
            }
            addSwatch(t) {
                const { values: e  } = this._parseLocalColor(t);
                if (e) {
                    const { _swatchColors: t1 , _root: o  } = this, n = A(...e), s = r(`<button type="button" style="--pcr-color: ${n.toRGBA().toString(0)}" aria-label="${this._t("btn:swatch")}"/>`);
                    return o.swatches.appendChild(s), t1.push({
                        el: s,
                        color: n
                    }), this._eventBindings.push(i(s, "click", ()=>{
                        this.setHSVA(...n.toHSVA(), !0), this._emit("swatchselect", n), this._emit("change", n, "swatch", this);
                    })), !0;
                }
                return !1;
            }
            removeSwatch(t) {
                const e = this._swatchColors[t];
                if (e) {
                    const { el: o  } = e;
                    return this._root.swatches.removeChild(o), this._swatchColors.splice(t, 1), !0;
                }
                return !1;
            }
            applyColor(t = !1) {
                const { preview: e , button: o  } = this._root, n = this._color.toRGBA().toString(0);
                return e.lastColor.style.setProperty("--pcr-color", n), this.options.useAsButton || o.style.setProperty("--pcr-color", n), o.classList.remove("clear"), this._lastColor = this._color.clone(), this._initializingActive || t || this._emit("save", this._color), this;
            }
            destroy() {
                cancelAnimationFrame(this._setupAnimationFrame), this._eventBindings.forEach((t)=>s(...t)), Object.keys(this._components).forEach((t)=>this._components[t].destroy());
            }
            destroyAndRemove() {
                this.destroy();
                const { root: t , app: e  } = this._root;
                t.parentElement && t.parentElement.removeChild(t), e.parentElement.removeChild(e), Object.keys(this).forEach((t)=>this[t] = null);
            }
            hide() {
                return !!this.isOpen() && (this._root.app.classList.remove("visible"), this._emit("hide"), !0);
            }
            show() {
                return !this.options.disabled && !this.isOpen() && (this._root.app.classList.add("visible"), this._rePositioningPicker(), this._emit("show", this._color), this);
            }
            isOpen() {
                return this._root.app.classList.contains("visible");
            }
            setHSVA(t = 360, e = 0, o = 0, n = 1, i = !1) {
                const s = this._recalc;
                if (this._recalc = !1, t < 0 || t > 360 || e < 0 || e > 100 || o < 0 || o > 100 || n < 0 || n > 1) return !1;
                this._color = A(t, e, o, n);
                const { hue: r , opacity: a , palette: l  } = this._components;
                return r.update(t / 360), a.update(n), l.update(e / 100, 1 - o / 100), i || this.applyColor(), s && this._updateOutput(), this._recalc = s, !0;
            }
            setColor(t, e = !1) {
                if (null === t) return this._clearColor(e), !0;
                const { values: o , type: n  } = this._parseLocalColor(t);
                if (o) {
                    const t1 = n.toUpperCase(), { options: i  } = this._root.interaction, s = i.find((e)=>e.getAttribute("data-type") === t1);
                    if (s && !s.hidden) for (const t2 of i)t2.classList[t2 === s ? "add" : "remove"]("active");
                    return !!this.setHSVA(...o, e) && this.setColorRepresentation(t1);
                }
                return !1;
            }
            setColorRepresentation(t) {
                return t = t.toUpperCase(), !!this._root.interaction.options.find((e)=>e.getAttribute("data-type").startsWith(t) && !e.click());
            }
            getColorRepresentation() {
                return this._representation;
            }
            getColor() {
                return this._color;
            }
            getSelectedColor() {
                return this._lastColor;
            }
            getRoot() {
                return this._root;
            }
            disable() {
                return this.hide(), this.options.disabled = !0, this._root.button.classList.add("disabled"), this;
            }
            enable() {
                return this.options.disabled = !1, this._root.button.classList.remove("disabled"), this;
            }
        }
        return E(L, "utils", o), E(L, "version", "1.8.2"), E(L, "I18N_DEFAULTS", {
            "ui:dialog": "color picker dialog",
            "btn:toggle": "toggle color picker dialog",
            "btn:swatch": "color swatch",
            "btn:last-color": "use previous color",
            "btn:save": "Save",
            "btn:cancel": "Cancel",
            "btn:clear": "Clear",
            "aria:btn:save": "save and close",
            "aria:btn:cancel": "cancel and close",
            "aria:btn:clear": "clear and close",
            "aria:input": "color input field",
            "aria:palette": "color selection area",
            "aria:hue": "hue selection slider",
            "aria:opacity": "selection slider"
        }), E(L, "DEFAULT_OPTIONS", {
            appClass: null,
            theme: "classic",
            useAsButton: !1,
            padding: 8,
            disabled: !1,
            comparison: !0,
            closeOnScroll: !1,
            outputPrecision: 0,
            lockOpacity: !1,
            autoReposition: !0,
            container: "body",
            components: {
                interaction: {}
            },
            i18n: {},
            swatches: null,
            inline: !1,
            sliders: null,
            default: "#42445a",
            defaultRepresentation: null,
            position: "bottom-middle",
            adjustableNumbers: !0,
            showAlways: !1,
            closeWithKey: "Escape"
        }), E(L, "create", (t)=>new L(t)), e = e.default;
    })();
});
},

};
})();

// mount Chunks
(function () {
	runtime.installedChunks = {};
})();

// mount ModuleCache
(function () {
	runtime.moduleCache = {};
})();
(function () {
	runtime.checkById = function (obj, prop) {
		return Object.prototype.hasOwnProperty.call(obj, prop);
	};
})();
// mount PublicPath
(function () {
	runtime.publicPath = "/";
})();
// The require function
function __rspack_require__(moduleId) {
	var cachedModule = runtime.moduleCache[moduleId];
	if (cachedModule !== undefined) {
		return cachedModule.exports;
	}

	// Create a new module (and put it into the cache)
	var module = (runtime.moduleCache[moduleId] = {
		// no module.id needed
		// no module.loaded needed
		exports: {}
	});

	// TODO: should use runtime generator
	//---- hot require
	try {
		var execOptions = {
			id: moduleId,
			module: module,
			factory: runtime.installedModules[moduleId],
			require: __rspack_require__
		};
		module = execOptions.module;
		__rspack_require__.i.forEach(function (handler) {
			handler(execOptions);
		});
		execOptions.factory.call(
			module.exports,
			module,
			module.exports,
			execOptions.require.bind(runtime),
			runtime.__rspack_dynamic_require__ &&
				runtime.__rspack_dynamic_require__.bind(runtime),
			runtime
		);
	} catch (error) {
		module.error = error;
		throw error;
	}

	//------ other
	// this.installedModules[moduleId](
	// 	module,
	// 	module.exports,
	// 	this.__rspack_require__.bind(this),
	// 	this.__rspack_dynamic_require__ &&
	// 		this.__rspack_dynamic_require__.bind(this),
	//  runtime,
	// );

	return module.exports;
}

// mount require function
(function () {
	runtime.__rspack_require__ = __rspack_require__;
	// module execution interceptor
	runtime.__rspack_require__.i = [];
	// hasOwnProperty shorthand
	runtime.__rspack_require__.o = (obj, prop) =>
		Object.prototype.hasOwnProperty.call(obj, prop);
})();
// The register function
function __rspack_register__(chunkIds, modules, callback) {
	if (
		chunkIds.some(
			function (id) {
				return this.installedChunks[id] !== 0;
			}.bind(this)
		)
	) {
		for (moduleId in modules) {
			if (this.checkById(modules, moduleId)) {
				this.installedModules[moduleId] = modules[moduleId];
			}
		}
		if (callback) callback(this.__rspack_require__);
	}
	for (var i = 0; i < chunkIds.length; i++) {
		chunkId = chunkIds[i];
		if (
			this.checkById(this.installedChunks, chunkId) &&
			this.installedChunks[chunkId]
		) {
			this.installedChunks[chunkId][0]();
		}
		this.installedChunks[chunkId] = 0;
	}
}

// mount register function
(function () {
	runtime.__rspack_register__ = __rspack_register__;
})();
(function(){
runtime.__rspack_require__.chunkId = 'main'})();(function(){
runtime.__rspack_require__.p = '/'})();// hot runtime
(function () {
	var currentModuleData = {};
	var installedModules = runtime.moduleCache;

	// module and require creation
	var currentChildModule;
	var currentParents = [];

	// status
	var registeredStatusHandlers = [];
	var currentStatus = "idle";

	// while downloading
	// TODO: not needed in rspack temporary,
	// TODO: because we transfer all changed modules.
	var blockingPromises = 0;
	var blockingPromisesWaiting = [];

	// The update info
	var currentUpdateApplyHandlers;
	var queuedInvalidatedModules;

	runtime.__rspack_require__.hmrD = currentModuleData;
	runtime.__rspack_require__.i.push(function (options) {
		var module = options.module;
		var require = createRequire(options.require, options.id);
		module.hot = createModuleHotObject(options.id, module);
		module.parents = currentParents;
		module.children = [];
		currentParents = [];
		options.require = require;
	});

	runtime.__rspack_require__.hmrC = {};
	// TODO: useless
	runtime.__rspack_require__.hmrI = {};

	function createRequire(require, moduleId) {
		var me = installedModules[moduleId];
		if (!me) {
			return require;
		}
		var fn = function (request) {
			if (me.hot.active) {
				if (installedModules[request]) {
					var parents = installedModules[request].parents;
					if (parents.indexOf(moduleId) === -1) {
						parents.push(moduleId);
					}
				} else {
					currentParents = [moduleId];
					currentChildModule = request;
				}
				if (me.children.indexOf(request) === -1) {
					me.children.push(request);
				}
			} else {
				console.log(
					"[HMR] unexpected require(" +
						request +
						") from disposed module " +
						moduleId
				);
				currentParents = [];
			}
			return require(request);
		};
		var createPropertyDescriptor = function (name) {
			return {
				configurable: true,
				enumerable: true,
				get: function () {
					return require[name];
				},
				set: function (value) {
					require[name] = value;
				}
			};
		};
		for (var name in require) {
			if (Object.prototype.hasOwnProperty.call(require, name) && name !== "e") {
				Object.defineProperty(fn, name, createPropertyDescriptor(name));
			}
		}

		fn.e = function (chunkId) {
			return trackBlockingPromise(require.e(chunkId));
		};

		return fn;
	}

	function createModuleHotObject(moduleId, me) {
		var _main = currentChildModule !== moduleId;
		var hot = {
			_acceptedDependencies: {},
			_acceptedErrorHandlers: {},
			_declinedDependencies: {},
			_selfAccepted: false,
			_selfDeclined: false,
			_selfInvalidated: false,
			_disposeHandlers: [],
			_main: _main,
			_requireSelf: function () {
				currentParents = me.parents.slice();
				currentChildModule = _main ? undefined : moduleId;
				runtime.__rspack_require__(moduleId);
			},
			active: true,
			accept: function (dep, callback, errorHandler) {
				if (dep === undefined) {
					hot._selfAccepted = true;
				} else if (typeof dep === "function") {
					hot._selfAccepted = dep;
				} else if (typeof dep === "object" && dep !== null) {
					for (var i = 0; i < dep.length; i++) {
						hot._acceptedDependencies[dep[i]] = callback || function () {};
						hot._acceptedErrorHandlers[dep[i]] = errorHandler;
					}
				} else {
					hot._acceptedDependencies[dep] = callback || function () {};
					hot._acceptedErrorHandlers[dep] = errorHandler;
				}
			},
			decline: function (dep) {
				if (dep === undefined) {
					hot._selfDeclined = true;
				} else if (typeof dep === "object" && dep !== null) {
					for (var i = 0; i < dep.length; i++) {
						hot._declinedDependencies[dep[i]] = true;
					}
				} else {
					hot._declinedDependencies[dep] = true;
				}
			},
			dispose: function (callback) {
				hot._disposeHandlers.push(callback);
			},
			addDisposeHandler: function (callback) {
				hot._disposeHandlers.push(callback);
			},
			removeDisposeHandler: function (callback) {
				var idx = hot._disposeHandlers.indexOf(callback);
				if (idx > 0) {
					hot._disposeHandlers.splice(idx, 1);
				}
			},
			invalidate: function () {
				this._selfInvalidated = true;
				switch (currentStatus) {
					case "idle":
						// TODO: useless
						currentUpdateApplyHandlers = [];
						Object.keys(runtime.__rspack_require__.hmrI).forEach(function (
							key
						) {
							runtime.__rspack_require__.hmrI[key](
								moduleId,
								currentUpdateApplyHandlers
							);
						});
						setStatus("ready");
						break;
					case "ready":
						Object.keys(runtime.__rspack_require__.hmrI).forEach(function (
							key
						) {
							runtime.__rspack_require__.hmrI[key](
								moduleId,
								currentUpdateApplyHandlers
							);
						});
						break;
					case "prepare":
					case "check":
					case "dispose":
					case "apply":
						(queuedInvalidatedModules = queuedInvalidatedModules || []).push(
							moduleId
						);
						break;
					default:
						break;
				}
			},
			check: hotCheck,
			apply: hotApply,
			status: function (l) {
				if (!l) {
					return currentStatus;
				}
				registeredStatusHandlers.push(l);
			},
			addStatusHandler: function (l) {
				registeredStatusHandlers.push(l);
			},
			removeStatusHandler: function (l) {
				var idx = registeredStatusHandlers.indexOf(l);
				if (idx >= 0) registeredStatusHandlers.splice(idx, 1);
			},
			data: currentModuleData[moduleId]
		};
		currentChildModule = undefined;
		return hot;
	}

	function setStatus(newStats) {
		currentStatus = newStats;
		var results = [];
		for (var i = 0; i < registeredStatusHandlers.length; i++) {
			results[i] = registeredStatusHandlers[i].call(null, newStats);
		}
		return Promise.all(results);
	}

	function unblock() {
		if (--blockingPromises === 0) {
			setStatus("ready").then(function () {
				if (blockingPromises === 0) {
					var list = blockingPromisesWaiting;
					blockingPromisesWaiting = [];
					for (var i = 0; i < list.length; i++) {
						list[i]();
					}
				}
			});
		}
	}

	function trackBlockingPromise(promise) {
		switch (currentStatus) {
			case "ready":
				setStatus("prepare");
			case "prepare":
				blockingPromises++;
				promise.then(unblock, unblock);
				return promise;
			default:
				return promise;
		}
	}

	function waitForBlockingPromises(fn) {
		if (blockingPromises === 0) {
			return fn();
		}
		return new Promise(function (resolve) {
			blockingPromisesWaiting.push(function () {
				resolve(fn());
			});
		});
	}

	function hotCheck(applyOnUpdate) {
		if (currentStatus !== "idle") {
			throw new Error("check() is only allowed in idle status");
		}
		return setStatus("check")
			.then(runtime.__rspack_require__.hmrM)
			.then(function (update) {
				if (!update) {
					return setStatus(applyInvalidatedModules() ? "ready" : "idle").then(
						function () {
							return null;
						}
					);
				}

				return setStatus("prepare").then(function () {
					// var updatedModules = [];
					// TODO: updatedModule should removed after hash
					var updatedModules = update.updatedModule;
					currentUpdateApplyHandlers = [];
					return Promise.all(
						// TODO: update.c, .r, .m is useless now.
						Object.keys(runtime.__rspack_require__.hmrC).reduce(function (
							promises,
							key
						) {
							runtime.__rspack_require__.hmrC[key](
								update.c,
								update.r,
								update.m,
								promises,
								currentUpdateApplyHandlers,
								updatedModules
							);
							return promises;
						},
						[])
					).then(function () {
						return waitForBlockingPromises(function () {
							if (applyOnUpdate) {
								return internalApply(applyOnUpdate);
							} else {
								return setStatus("ready").then(function () {
									return updatedModules;
								});
							}
						});
					});
				});
			});
	}

	function hotApply(options) {
		if (currentStatus !== "ready") {
			return Promise.resolve().then(function () {
				throw Error(
					"apply() is only allowed in ready status (state: " +
						currentStatus +
						")"
				);
			});
		}
		return internalApply(options);
	}

	function internalApply(options) {
		options = options || {};
		applyInvalidatedModules();
		var results = currentUpdateApplyHandlers.map(function (handler) {
			return handler(options);
		});
		currentUpdateApplyHandlers = undefined;
		var errors = results
			.map(function (r) {
				return r.errors;
			})
			.filter(Boolean);

		if (errors.length > 0) {
			return setStatus("abort").then(function () {
				throw errors[0];
			});
		}

		var disposePromise = setStatus("dispose");

		results.forEach(function (result) {
			if (result.dispose) {
				result.dispose();
			}
		});

		var applyPromise = setStatus("apply");

		var error;
		var reportError = function (err) {
			if (!error) {
				error = err;
			}
		};

		var outdatedModules = [];
		results.forEach(function (result) {
			if (result.apply) {
				var modules = result.apply(reportError);
				if (modules) {
					for (var i = 0; i < modules.length; i++) {
						outdatedModules.push(modules[i]);
					}
				}
			}
		});

		return Promise.all([disposePromise, applyPromise]).then(function () {
			if (error) {
				return setStatus("fail").then(function () {
					throw error;
				});
			}

			if (queuedInvalidatedModules) {
				return internalApply(options).then(function (list) {
					outdatedModules.forEach(function (moduleId) {
						if (list.indexOf(moduleId) < 0) {
							list.push(moduleId);
						}
					});
					return list;
				});
			}

			return setStatus("idle").then(function () {
				return outdatedModules;
			});
		});
	}

	function applyInvalidatedModules() {
		if (queuedInvalidatedModules) {
			if (!currentUpdateApplyHandlers) {
				currentUpdateApplyHandlers = [];
			}
			Object.keys(runtime.__rspack_require__.hmrI).forEach(function (key) {
				queuedInvalidatedModules.forEach(function (moduleId) {
					runtime.__rspack_require__.hmrI[key](
						moduleId,
						currentUpdateApplyHandlers
					);
				});
			});
			queuedInvalidatedModules = undefined;
			return true;
		}
	}
})();
(() => {
	var inProgress = {};
	// data-webpack is not used as build has no uniqueName
	// loadScript function to load a script via script tag
	runtime.__rspack_require__.l = (content, done, key, chunkId) => {
		// if (inProgress[url]) {
		// 	inProgress[url].push(done);
		// 	return;
		// }
		var script, needAttach;
		if (key !== undefined) {
			var scripts = document.getElementsByTagName("script");
			for (var i = 0; i < scripts.length; i++) {
				var s = scripts[i];
				// if (s.getAttribute("src") == url) {
				// 	script = s;
				// 	break;
				// }
				if (s.text == content) {
					script = s;
					break;
				}
			}
		}
		if (!script) {
			needAttach = true;
			script = document.createElement("script");

			script.charset = "utf-8";
			script.timeout = 120;
			script.id = "hot-script";
			// if (__webpack_require__.nc) {
			// 	script.setAttribute("nonce", __webpack_require__.nc);
			// }

			// script.src = url;
			script.text = content;
		}
		// inProgress[url] = [done];
		inProgress[content] = [done];
		var onScriptComplete = (prev, event) => {
			// avoid mem leaks in IE.
			script.onerror = script.onload = null;
			clearTimeout(timeout);
			// var doneFns = inProgress[url];
			// delete inProgress[url];
			var doneFns = inProgress[content];
			delete inProgress[content];
			script.parentNode && script.parentNode.removeChild(script);
			doneFns && doneFns.forEach(fn => fn(event));
			if (prev) return prev(event);
		};
		var timeout = setTimeout(
			onScriptComplete.bind(null, undefined, {
				type: "timeout",
				target: script
			}),
			120000
		);
		script.onerror = onScriptComplete.bind(null, script.onerror);
		script.onload = onScriptComplete.bind(null, script.onload);
		needAttach && document.head.appendChild(script);
	};
})();
(function () {
	var installedChunks = (runtime.__rspack_require__.hmrS_jsonp = runtime
		.__rspack_require__.hmrS_jsonp || {
		main: 0
	});

	var currentUpdatedModulesList;
	var waitingUpdateResolves = {};
	function loadUpdateChunk(chunkId, updatedModulesList, content) {
		currentUpdatedModulesList = updatedModulesList;
		return new Promise((resolve, reject) => {
			waitingUpdateResolves[chunkId] = resolve;
			// start update chunk loading
			// var url = __webpack_require__.p + __webpack_require__.hu(chunkId);
			// create error before stack unwound to get useful stacktrace later
			var error = new Error();
			var loadingEnded = event => {
				if (waitingUpdateResolves[chunkId]) {
					waitingUpdateResolves[chunkId] = undefined;
					var errorType =
						event && (event.type === "load" ? "missing" : event.type);
					var realSrc = event && event.target && event.target.src;
					error.message =
						"Loading hot update chunk " +
						chunkId +
						" failed.\n(" +
						errorType +
						": " +
						realSrc +
						")";
					error.name = "ChunkLoadError";
					error.type = errorType;
					error.request = realSrc;
					reject(error);
				}
			};
			runtime.__rspack_require__.l(content, loadingEnded);
		});
	}

	self["hotUpdate"] = (chunkId, moreModules, runtime) => {
		for (var moduleId in moreModules) {
			if (__rspack_runtime__.__rspack_require__.o(moreModules, moduleId)) {
				currentUpdate[moduleId] = moreModules[moduleId];
				if (currentUpdatedModulesList) currentUpdatedModulesList.push(moduleId);
			}
		}
		if (runtime) currentUpdateRuntime.push(runtime);
		if (waitingUpdateResolves[chunkId]) {
			waitingUpdateResolves[chunkId]();
			waitingUpdateResolves[chunkId] = undefined;
			var tag = document.getElementById("hot-script");
			tag && tag.parentNode && tag.parentNode.removeChild(tag);
		}
	};

	var currentUpdateChunks;
	var currentUpdate;
	var currentUpdateRemovedChunks;
	var currentUpdateRuntime;
	function applyHandler(options) {
		currentUpdateChunks = undefined;
		function getAffectedModuleEffects(updateModuleId) {
			var outdatedModules = [updateModuleId];
			var outdatedDependencies = {};
			var queue = outdatedModules.map(function (id) {
				return {
					chain: [id],
					id: id
				};
			});
			while (queue.length > 0) {
				var queueItem = queue.pop();
				var moduleId = queueItem.id;
				var chain = queueItem.chain;
				var module = runtime.moduleCache[moduleId];
				if (
					!module ||
					(module.hot._selfAccepted && !module.hot._selfInvalidated)
				) {
					continue;
				}

				if (module.hot._selfDeclined) {
					return {
						type: "self-declined",
						chain: chain,
						moduleId: moduleId
					};
				}

				if (module.hot._main) {
					return {
						type: "unaccepted",
						chain: chain,
						moduleId: moduleId
					};
				}

				for (var i = 0; i < module.parents.length; i++) {
					var parentId = module.parents[i];
					var parent = runtime.moduleCache[parentId];
					if (!parent) {
						continue;
					}
					if (parent.hot._declinedDependencies[moduleId]) {
						return {
							type: "declined",
							chain: chain.concat([parentId]),
							moduleId: moduleId,
							parentId: parentId
						};
					}
					if (outdatedModules.indexOf(parentId) !== -1) {
						continue;
					}
					if (parent.hot._acceptedDependencies[moduleId]) {
						if (!outdatedDependencies[parentId]) {
							outdatedDependencies[parentId] = [];
						}
						addAllToSet(outdatedDependencies[parentId], [moduleId]);
						continue;
					}
					delete outdatedDependencies[parentId];
					outdatedModules.push(parentId);
					queue.push({
						chain: chain.concat([parentId]),
						id: parentId
					});
				}
			}

			return {
				type: "accepted",
				moduleId: updateModuleId,
				outdatedModules: outdatedModules,
				outdatedDependencies: outdatedDependencies
			};
		}

		function addAllToSet(a, b) {
			for (var i = 0; i < b.length; i++) {
				var item = b[i];
				if (a.indexOf(item) === -1) a.push(item);
			}
		}

		var outdatedDependencies = {};
		var outdatedModules = [];
		var appliedUpdate = {};

		var warnUnexpectedRequire = function warnUnexpectedRequire(module) {
			console.warn(
				"[HMR] unexpected require(" + module.id + ") to disposed module"
			);
		};

		for (var moduleId in currentUpdate) {
			if (runtime.__rspack_require__.o(currentUpdate, moduleId)) {
				var newModuleFactory = currentUpdate[moduleId];
				var result;
				if (newModuleFactory) {
					result = getAffectedModuleEffects(moduleId);
				} else {
					result = {
						type: "disposed",
						moduleId: moduleId
					};
				}
				var abortError = false;
				var doApply = false;
				var doDispose = false;
				var chainInfo = "";
				if (result.chain) {
					chainInfo = "\nUpdate propagation: " + result.chain.join(" -> ");
				}
				switch (result.type) {
					case "self-declined":
						if (options.onDeclined) options.onDeclined(result);
						if (!options.ignoreDeclined)
							abortError = new Error(
								"Aborted because of self decline: " +
									result.moduleId +
									chainInfo
							);
						break;
					case "declined":
						if (options.onDeclined) options.onDeclined(result);
						if (!options.ignoreDeclined)
							abortError = new Error(
								"Aborted because of declined dependency: " +
									result.moduleId +
									" in " +
									result.parentId +
									chainInfo
							);
						break;
					case "unaccepted":
						if (options.onUnaccepted) options.onUnaccepted(result);
						if (!options.ignoreUnaccepted)
							abortError = new Error(
								"Aborted because " + moduleId + " is not accepted" + chainInfo
							);
						break;
					case "accepted":
						if (options.onAccepted) options.onAccepted(result);
						doApply = true;
						break;
					case "disposed":
						if (options.onDisposed) options.onDisposed(result);
						doDispose = true;
						break;
					default:
						throw new Error("Unexception type " + result.type);
				}
				if (abortError) {
					return {
						error: abortError
					};
				}
				if (doApply) {
					appliedUpdate[moduleId] = newModuleFactory;
					addAllToSet(outdatedModules, result.outdatedModules);
					for (moduleId in result.outdatedDependencies) {
						if (
							runtime.__rspack_require__.o(
								result.outdatedDependencies,
								moduleId
							)
						) {
							if (!outdatedDependencies[moduleId])
								outdatedDependencies[moduleId] = [];
							addAllToSet(
								outdatedDependencies[moduleId],
								result.outdatedDependencies[moduleId]
							);
						}
					}
				}
				if (doDispose) {
					addAllToSet(outdatedModules, [result.moduleId]);
					appliedUpdate[moduleId] = warnUnexpectedRequire;
				}
			}
		}
		currentUpdate = undefined;

		var outdatedSelfAcceptedModules = [];
		for (var j = 0; j < outdatedModules.length; j++) {
			var outdatedModuleId = outdatedModules[j];
			var module = runtime.moduleCache[outdatedModuleId];
			if (
				module &&
				(module.hot._selfAccepted || module.hot._main) &&
				// removed self-accepted modules should not be required
				appliedUpdate[outdatedModuleId] !== warnUnexpectedRequire &&
				// when called invalidate self-accepting is not possible
				!module.hot._selfInvalidated
			) {
				outdatedSelfAcceptedModules.push({
					module: outdatedModuleId,
					require: module.hot._requireSelf,
					errorHandler: module.hot._selfAccepted
				});
			}
		}

		var moduleOutdatedDependencies;
		return {
			dispose: function () {
				currentUpdateRemovedChunks.forEach(function (chunkId) {
					delete installedChunks[chunkId];
				});
				currentUpdateRemovedChunks = undefined;

				var idx;
				var queue = outdatedModules.slice();
				while (queue.length > 0) {
					var moduleId = queue.pop();
					var module = runtime.moduleCache[moduleId];
					if (!module) continue;

					var data = {};

					// Call dispose handlers
					var disposeHandlers = module.hot._disposeHandlers;
					for (j = 0; j < disposeHandlers.length; j++) {
						disposeHandlers[j].call(null, data);
					}
					runtime.__rspack_require__.hmrD[moduleId] = data;

					module.hot.active = false;

					delete runtime.moduleCache[moduleId];

					delete outdatedDependencies[moduleId];

					for (j = 0; j < module.children.length; j++) {
						var child = runtime.moduleCache[module.children[j]];
						if (!child) continue;
						idx = child.parents.indexOf(moduleId);
						if (idx >= 0) {
							child.parents.splice(idx, 1);
						}
					}
				}

				var dependency;
				for (var outdatedModuleId in outdatedDependencies) {
					if (
						runtime.__rspack_require__.o(outdatedDependencies, outdatedModuleId)
					) {
						module = runtime.moduleCache[outdatedModuleId];
						if (module) {
							moduleOutdatedDependencies =
								outdatedDependencies[outdatedModuleId];
							for (j = 0; j < moduleOutdatedDependencies.length; j++) {
								dependency = moduleOutdatedDependencies[j];
								idx = module.children.indexOf(dependency);
								if (idx >= 0) module.children.splice(idx, 1);
							}
						}
					}
				}
			},
			apply: function (reportError) {
				// insert new code
				for (var updateModuleId in appliedUpdate) {
					if (runtime.__rspack_require__.o(appliedUpdate, updateModuleId)) {
						runtime.installedModules[updateModuleId] =
							appliedUpdate[updateModuleId];
					}
				}

				// run new runtime modules
				for (var i = 0; i < currentUpdateRuntime.length; i++) {
					currentUpdateRuntime[i](runtime.__rspack_require__);
				}

				// call accept handlers
				for (var outdatedModuleId in outdatedDependencies) {
					if (
						runtime.__rspack_require__.o(outdatedDependencies, outdatedModuleId)
					) {
						var module = runtime.moduleCache[outdatedModuleId];
						if (module) {
							moduleOutdatedDependencies =
								outdatedDependencies[outdatedModuleId];
							var callbacks = [];
							var errorHandlers = [];
							var dependenciesForCallbacks = [];
							for (var j = 0; j < moduleOutdatedDependencies.length; j++) {
								var dependency = moduleOutdatedDependencies[j];
								var acceptCallback =
									module.hot._acceptedDependencies[dependency];
								var errorHandler =
									module.hot._acceptedErrorHandlers[dependency];
								if (acceptCallback) {
									if (callbacks.indexOf(acceptCallback) !== -1) continue;
									callbacks.push(acceptCallback);
									errorHandlers.push(errorHandler);
									dependenciesForCallbacks.push(dependency);
								}
							}
							for (var k = 0; k < callbacks.length; k++) {
								try {
									callbacks[k].call(null, moduleOutdatedDependencies);
								} catch (err) {
									if (typeof errorHandlers[k] === "function") {
										try {
											errorHandlers[k](err, {
												moduleId: outdatedModuleId,
												dependencyId: dependenciesForCallbacks[k]
											});
										} catch (err2) {
											if (options.onErrored) {
												options.onErrored({
													type: "accept-error-handler-errored",
													moduleId: outdatedModuleId,
													dependencyId: dependenciesForCallbacks[k],
													error: err2,
													originalError: err
												});
											}
											if (!options.ignoreErrored) {
												reportError(err2);
												reportError(err);
											}
										}
									} else {
										if (options.onErrored) {
											options.onErrored({
												type: "accept-errored",
												moduleId: outdatedModuleId,
												dependencyId: dependenciesForCallbacks[k],
												error: err
											});
										}
										if (!options.ignoreErrored) {
											reportError(err);
										}
									}
								}
							}
						}
					}
				}

				// Load self accepted modules
				for (var o = 0; o < outdatedSelfAcceptedModules.length; o++) {
					var item = outdatedSelfAcceptedModules[o];
					var moduleId = item.module;
					try {
						item.require(moduleId);
					} catch (err) {
						if (typeof item.errorHandler === "function") {
							try {
								item.errorHandler(err, {
									moduleId: moduleId,
									module: runtime.moduleCache[moduleId]
								});
							} catch (err2) {
								if (options.onErrored) {
									options.onErrored({
										type: "self-accept-error-handler-errored",
										moduleId: moduleId,
										error: err2,
										originalError: err
									});
								}
								if (!options.ignoreErrored) {
									reportError(err2);
									reportError(err);
								}
							}
						} else {
							if (options.onErrored) {
								options.onErrored({
									type: "self-accept-errored",
									moduleId: moduleId,
									error: err
								});
							}
							if (!options.ignoreErrored) {
								reportError(err);
							}
						}
					}
				}

				return outdatedModules;
			}
		};
	}

	runtime.__rspack_require__.hmrI.jsonp = function (moduleId, applyHandlers) {
		if (!currentUpdate) {
			currentUpdate = {};
			currentUpdateRuntime = [];
			currentUpdateRemovedChunks = [];
			applyHandlers.push(applyHandler);
		}
		if (!runtime.__rspack_require__.o(currentUpdate, moduleId)) {
			currentUpdate[moduleId] = runtime.installedModules[moduleId];
		}
	};

	// TODO: fetch is not needed
	runtime.__rspack_require__.hmrC.jsonp = function (
		chunkIds,
		removedChunks,
		removedModules,
		promises,
		applyHandlers,
		// updatedModulesList,
		updatedModules
	) {
		applyHandlers.push(applyHandler);
		currentUpdateChunks = {};
		currentUpdateRemovedChunks = removedChunks;
		currentUpdate = removedModules.reduce(function (obj, key) {
			obj[key] = false;
			return obj;
		}, {});
		currentUpdateRuntime = [];
		chunkIds.forEach(function (chunkId) {
			if (
				runtime.__rspack_require__.o(installedChunks, chunkId) &&
				installedChunks[chunkId] !== undefined
			) {
				// TODO: use load script after hash.
				// promises.push(loadUpdateChunk(chunkId, updatedModulesList));
				var updatedModulesList = [updatedModules.uri];
				promises.push(
					loadUpdateChunk(chunkId, updatedModulesList, updatedModules.content)
				);

				currentUpdateChunks[chunkId] = true;
			} else {
				currentUpdateChunks[chunkId] = false;
			}
		});
	};
})();
(function () {
	function _getRequireCache(nodeInterop) {
		if (typeof WeakMap !== "function") return null;

		var cacheBabelInterop = new WeakMap();
		var cacheNodeInterop = new WeakMap();
		return (_getRequireCache = function (nodeInterop) {
			return nodeInterop ? cacheNodeInterop : cacheBabelInterop;
		})(nodeInterop);
	}

	runtime.interopRequire = function (obj, nodeInterop) {
		if (!nodeInterop && obj && obj.__esModule) {
			return obj;
		}

		if (
			obj === null ||
			(typeof obj !== "object" && typeof obj !== "function")
		) {
			return { default: obj };
		}

		var cache = _getRequireCache(nodeInterop);
		if (cache && cache.has(obj)) {
			return cache.get(obj);
		}

		var newObj = {};
		var hasPropertyDescriptor =
			Object.defineProperty && Object.getOwnPropertyDescriptor;
		for (var key in obj) {
			if (key !== "default" && Object.prototype.hasOwnProperty.call(obj, key)) {
				var desc = hasPropertyDescriptor
					? Object.getOwnPropertyDescriptor(obj, key)
					: null;
				if (desc && (desc.get || desc.set)) {
					Object.defineProperty(newObj, key, desc);
				} else {
					newObj[key] = obj[key];
				}
			}
		}
		newObj.default = obj;
		if (cache) {
			cache.set(obj, newObj);
		}
		return newObj;
	};
})();
self["__rspack_runtime__"].__rspack_require__("./index.js");})()