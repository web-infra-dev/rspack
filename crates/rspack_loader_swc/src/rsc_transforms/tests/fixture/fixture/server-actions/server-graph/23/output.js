import { registerServerReference } from "react-server-dom-rspack/server";
import { encryptActionBoundArgs, decryptActionBoundArgs } from "react-server-dom-rspack/server";
export const $$RSC_SERVER_ACTION_0 = async function action($$ACTION_CLOSURE_BOUND, a, b, c, d) {
    var [$$ACTION_ARG_0] = await decryptActionBoundArgs("7cd7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", $$ACTION_CLOSURE_BOUND);
    console.log(a, b, $$ACTION_ARG_0, c, d);
};
registerServerReference($$RSC_SERVER_ACTION_0, "7cd7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", null);
export const $$RSC_SERVER_ACTION_1 = async function action2($$ACTION_CLOSURE_BOUND, a, b, c, d) {
    var [$$ACTION_ARG_0] = await decryptActionBoundArgs("7cbe8d67bb01f9fc1199f26e7d4bbc812d5446fe5337d20608749bb1de5f3e8caa", $$ACTION_CLOSURE_BOUND);
    console.log(a, b, $$ACTION_ARG_0, c, d);
};
registerServerReference($$RSC_SERVER_ACTION_1, "7cbe8d67bb01f9fc1199f26e7d4bbc812d5446fe5337d20608749bb1de5f3e8caa", null);
export default function Page({ foo, x, y }) {
    var action = $$RSC_SERVER_ACTION_0.bind(null, encryptActionBoundArgs("7cd7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", x));
    action.bind(null, foo[0], foo[1], foo.x, foo[y]);
    const action2 = $$RSC_SERVER_ACTION_1.bind(null, encryptActionBoundArgs("7cbe8d67bb01f9fc1199f26e7d4bbc812d5446fe5337d20608749bb1de5f3e8caa", x));
    action2.bind(null, foo[0], foo[1], foo.x, foo[y]);
}
