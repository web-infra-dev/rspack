import { registerServerReference } from "react-server-dom-rspack/server";
export async function a() {}
export async function b() {}
export async function c() {}
function d() {}
export const $$RSC_SERVER_ACTION_0 = async function e() {};
registerServerReference($$RSC_SERVER_ACTION_0, "00d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", null);
function Foo() {
    var e = $$RSC_SERVER_ACTION_0;
}
import { ensureServerActions } from "react-server-dom-rspack/server";
ensureServerActions([
    a,
    b,
    c
]);
registerServerReference(a, "00e219eb46414df08b42d5a3ec301215fe7aea228d8546617bc50d5e0bc56177d4", null);
registerServerReference(b, "0096290f7a1b362cb7c1fada321538d6a51a2fce469fc9577f1bcc28ef072f5916", null);
registerServerReference(c, "002e67d26589d4fb0b226a00a31710c1c72daccbd60a7cbea3b7f7b6158f349f39", null);
