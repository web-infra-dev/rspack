
(self.webpackChunkapp4 = self.webpackChunkapp4 || []).push([
	["874"],
	{
		225: function (e, t, n) {
			(n.r(t), n.d(t, { default: () => l, loadApp: () => r }));
			var o = n(793);
			const a = new o.Z({
					target: document.querySelector("#app_04"),
					props: { name: "world" }
				}),
				r = e =>
					new o.Z({
						target: document.querySelector("#app_04"),
						props: { name: "world" }
					});
			window.app = a;
			const l = a;
		},
		793: function (e, t, n) {
			n.d(t, { Z: () => p });
			var o = n(492);
			function a(e) {
				let t, n, a, r;
				return {
					c() {
						((t = (0, o.bGB)("h1")),
							(n = (0, o.fLW)("Hello From Svelte ")),
							(a = (0, o.fLW)(e[0])),
							(r = (0, o.fLW)("!")),
							(0, o.Ljt)(t, "class", "svelte-1ucbz36"));
					},
					m(e, l) {
						((0, o.$Tr)(e, t, l),
							(0, o.R3I)(t, n),
							(0, o.R3I)(t, a),
							(0, o.R3I)(t, r));
					},
					p(e, [t]) {
						1 & t && (0, o.rTO)(a, e[0]);
					},
					i: o.ZTd,
					o: o.ZTd,
					d(e) {
						e && (0, o.ogt)(t);
					}
				};
			}
			function r(e, t, n) {
				let { name: o } = t;
				return (
					window.addEventListener("change-name", e => {
						(console.log(e), void 0 !== e.detail && n(0, (o = e.detail.name)));
					}),
					(e.$$set = e => {
						"name" in e && n(0, (o = e.name));
					}),
					[o]
				);
			}
			(n(374), n(535));
			class l extends o.f_C {
				constructor(e) {
					(super(), (0, o.S1n)(this, e, r, a, o.N8, { name: 0 }));
				}
			}
			const p = l;
		},
		535: function (e, t, n) {
			e.exports = {};
		}
	}
]);
