
(self.webpackChunkapp4 = self.webpackChunkapp4 || []).push([
	["474"],
	{
		966: function (e, t, n) {
			(n.r(t), n.d(t, { default: () => r }));
			var o = n(793);
			const r = (e, t) =>
				new o.Z({
					target: document.querySelector(`#${e}`),
					props: { name: t }
				});
		},
		793: function (e, t, n) {
			n.d(t, { Z: () => c });
			var o = n(492);
			function r(e) {
				let t, n, r, a;
				return {
					c() {
						((t = (0, o.bGB)("h1")),
							(n = (0, o.fLW)("Hello From Svelte ")),
							(r = (0, o.fLW)(e[0])),
							(a = (0, o.fLW)("!")),
							(0, o.Ljt)(t, "class", "svelte-1ucbz36"));
					},
					m(e, s) {
						((0, o.$Tr)(e, t, s),
							(0, o.R3I)(t, n),
							(0, o.R3I)(t, r),
							(0, o.R3I)(t, a));
					},
					p(e, [t]) {
						1 & t && (0, o.rTO)(r, e[0]);
					},
					i: o.ZTd,
					o: o.ZTd,
					d(e) {
						e && (0, o.ogt)(t);
					}
				};
			}
			function a(e, t, n) {
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
			class s extends o.f_C {
				constructor(e) {
					(super(), (0, o.S1n)(this, e, a, r, o.N8, { name: 0 }));
				}
			}
			const c = s;
		},
		535: function (e, t, n) {
			e.exports = {};
		}
	}
]);
