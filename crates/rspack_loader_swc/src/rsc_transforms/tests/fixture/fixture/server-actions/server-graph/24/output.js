import { registerServerReference } from "react-server-dom-rspack/server";
import { encryptActionBoundArgs, decryptActionBoundArgs } from "react-server-dom-rspack/server";
export const $$RSC_SERVER_ACTION_0 = async function action($$ACTION_CLOSURE_BOUND, a, b, c, { d }) {
    var [$$ACTION_ARG_0] = await decryptActionBoundArgs("7cd7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", $$ACTION_CLOSURE_BOUND);
    console.log(a, b, $$ACTION_ARG_0, d);
};
registerServerReference($$RSC_SERVER_ACTION_0, "7cd7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", null);
export default function Page({ foo, x, y }) {
    var action = $$RSC_SERVER_ACTION_0.bind(null, encryptActionBoundArgs("7cd7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", foo));
}
