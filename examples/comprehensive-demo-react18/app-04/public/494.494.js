
(self.webpackChunkcomprehensive_demo_react18_app_04 =
	self.webpackChunkcomprehensive_demo_react18_app_04 || []).push([
	[494],
	{
		494: (t, e, n) => {
			function o() {}
			function r(t) {
				return t();
			}
			function c() {
				return Object.create(null);
			}
			function s(t) {
				t.forEach(r);
			}
			function i(t) {
				return "function" == typeof t;
			}
			function a(t, e) {
				return t != t
					? e == e
					: t !== e || (t && "object" == typeof t) || "function" == typeof t;
			}
			function u(t) {
				return 0 === Object.keys(t).length;
			}
			(n.d(e, {
				$Tr: () => $,
				Ljt: () => g,
				N8: () => a,
				R3I: () => f,
				S1n: () => M,
				ZTd: () => o,
				bGB: () => p,
				fLW: () => m,
				f_C: () => q,
				ogt: () => h,
				rTO: () => _
			}),
				new Set());
			let l,
				d = !1;
			function f(t, e) {
				t.appendChild(e);
			}
			function $(t, e, n) {
				t.insertBefore(e, n || null);
			}
			function h(t) {
				t.parentNode && t.parentNode.removeChild(t);
			}
			function p(t) {
				return document.createElement(t);
			}
			function m(t) {
				return document.createTextNode(t);
			}
			function g(t, e, n) {
				null == n
					? t.removeAttribute(e)
					: t.getAttribute(e) !== n && t.setAttribute(e, n);
			}
			function _(t, e) {
				((e = "" + e), t.data !== e && (t.data = e));
			}
			function b(t) {
				l = t;
			}
			new Map();
			const y = [],
				k = [];
			let w = [];
			const x = [],
				C = Promise.resolve();
			let E = !1;
			function v(t) {
				w.push(t);
			}
			const T = new Set();
			let O = 0;
			function S() {
				if (0 !== O) return;
				const t = l;
				do {
					try {
						for (; O < y.length; ) {
							const t = y[O];
							(O++, b(t), N(t.$$));
						}
					} catch (t) {
						throw ((y.length = 0), (O = 0), t);
					}
					for (b(null), y.length = 0, O = 0; k.length; ) k.pop()();
					for (let t = 0; t < w.length; t += 1) {
						const e = w[t];
						T.has(e) || (T.add(e), e());
					}
					w.length = 0;
				} while (y.length);
				for (; x.length; ) x.pop()();
				((E = !1), T.clear(), b(t));
			}
			function N(t) {
				if (null !== t.fragment) {
					(t.update(), s(t.before_update));
					const e = t.dirty;
					((t.dirty = [-1]),
						t.fragment && t.fragment.p(t.ctx, e),
						t.after_update.forEach(v));
				}
			}
			const j = new Set();
			let A;
			function L(t, e) {
				const n = t.$$;
				null !== n.fragment &&
					((function (t) {
						const e = [],
							n = [];
						(w.forEach(o => (-1 === t.indexOf(o) ? e.push(o) : n.push(o))),
							n.forEach(t => t()),
							(w = e));
					})(n.after_update),
					s(n.on_destroy),
					n.fragment && n.fragment.d(e),
					(n.on_destroy = n.fragment = null),
					(n.ctx = []));
			}
			function M(t, e, n, a, u, f, $, p = [-1]) {
				const m = l;
				b(t);
				const g = (t.$$ = {
					fragment: null,
					ctx: [],
					props: f,
					update: o,
					not_equal: u,
					bound: c(),
					on_mount: [],
					on_destroy: [],
					on_disconnect: [],
					before_update: [],
					after_update: [],
					context: new Map(e.context || (m ? m.$$.context : [])),
					callbacks: c(),
					dirty: p,
					skip_bound: !1,
					root: e.target || m.$$.root
				});
				$ && $(g.root);
				let _ = !1;
				if (
					((g.ctx = n
						? n(t, e.props || {}, (e, n, ...o) => {
								const r = o.length ? o[0] : n;
								return (
									g.ctx &&
										u(g.ctx[e], (g.ctx[e] = r)) &&
										(!g.skip_bound && g.bound[e] && g.bound[e](r),
										_ &&
											(function (t, e) {
												(-1 === t.$$.dirty[0] &&
													(y.push(t),
													E || ((E = !0), C.then(S)),
													t.$$.dirty.fill(0)),
													(t.$$.dirty[(e / 31) | 0] |= 1 << e % 31));
											})(t, e)),
									n
								);
							})
						: []),
					g.update(),
					(_ = !0),
					s(g.before_update),
					(g.fragment = !!a && a(g.ctx)),
					e.target)
				) {
					if (e.hydrate) {
						d = !0;
						const t = (function (t) {
							return Array.from(t.childNodes);
						})(e.target);
						(g.fragment && g.fragment.l(t), t.forEach(h));
					} else g.fragment && g.fragment.c();
					(e.intro && (k = t.$$.fragment) && k.i && (j.delete(k), k.i(w)),
						(function (t, e, n, o) {
							const { fragment: c, after_update: a } = t.$$;
							(c && c.m(e, n),
								o ||
									v(() => {
										const e = t.$$.on_mount.map(r).filter(i);
										(t.$$.on_destroy ? t.$$.on_destroy.push(...e) : s(e),
											(t.$$.on_mount = []));
									}),
								a.forEach(v));
						})(t, e.target, e.anchor, e.customElement),
						(d = !1),
						S());
				}
				var k, w;
				b(m);
			}
			("undefined" != typeof window
				? window
				: "undefined" != typeof globalThis
					? globalThis
					: n.g,
				new Set([
					"allowfullscreen",
					"allowpaymentrequest",
					"async",
					"autofocus",
					"autoplay",
					"checked",
					"controls",
					"default",
					"defer",
					"disabled",
					"formnovalidate",
					"hidden",
					"inert",
					"ismap",
					"loop",
					"multiple",
					"muted",
					"nomodule",
					"novalidate",
					"open",
					"playsinline",
					"readonly",
					"required",
					"reversed",
					"selected"
				]),
				"function" == typeof HTMLElement &&
					(A = class extends HTMLElement {
						constructor() {
							(super(), this.attachShadow({ mode: "open" }));
						}
						connectedCallback() {
							const { on_mount: t } = this.$$;
							this.$$.on_disconnect = t.map(r).filter(i);
							for (const t in this.$$.slotted)
								this.appendChild(this.$$.slotted[t]);
						}
						attributeChangedCallback(t, e, n) {
							this[t] = n;
						}
						disconnectedCallback() {
							s(this.$$.on_disconnect);
						}
						$destroy() {
							(L(this, 1), (this.$destroy = o));
						}
						$on(t, e) {
							if (!i(e)) return o;
							const n = this.$$.callbacks[t] || (this.$$.callbacks[t] = []);
							return (
								n.push(e),
								() => {
									const t = n.indexOf(e);
									-1 !== t && n.splice(t, 1);
								}
							);
						}
						$set(t) {
							this.$$set &&
								!u(t) &&
								((this.$$.skip_bound = !0),
								this.$$set(t),
								(this.$$.skip_bound = !1));
						}
					}));
			class q {
				$destroy() {
					(L(this, 1), (this.$destroy = o));
				}
				$on(t, e) {
					if (!i(e)) return o;
					const n = this.$$.callbacks[t] || (this.$$.callbacks[t] = []);
					return (
						n.push(e),
						() => {
							const t = n.indexOf(e);
							-1 !== t && n.splice(t, 1);
						}
					);
				}
				$set(t) {
					this.$$set &&
						!u(t) &&
						((this.$$.skip_bound = !0),
						this.$$set(t),
						(this.$$.skip_bound = !1));
				}
			}
		}
	}
]);
