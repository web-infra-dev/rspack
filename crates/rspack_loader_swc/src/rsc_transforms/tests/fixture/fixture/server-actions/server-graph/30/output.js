import { registerServerReference } from "react-server-dom-rspack/server";
import { encryptActionBoundArgs, decryptActionBoundArgs } from "react-server-dom-rspack/server";
let a, f;
export const // FIXME: invalid transformation of hoisted functions (https://github.com/vercel/next.js/issues/57392)
// (remove output.js from `tsconfig.json#exclude` to see the error)
$$RSC_SERVER_ACTION_0 = async function action2($$ACTION_CLOSURE_BOUND, e) {
    var [$$ACTION_ARG_0, $$ACTION_ARG_1, $$ACTION_ARG_2, $$ACTION_ARG_3] = await decryptActionBoundArgs("60d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", $$ACTION_CLOSURE_BOUND);
    console.log(a, $$ACTION_ARG_0, $$ACTION_ARG_1, e, $$ACTION_ARG_2, $$ACTION_ARG_3);
};
registerServerReference($$RSC_SERVER_ACTION_0, "60d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", null);
export const $$RSC_SERVER_ACTION_1 = async function action3($$ACTION_CLOSURE_BOUND, e) {
    var [$$ACTION_ARG_0, $$ACTION_ARG_1, $$ACTION_ARG_2] = await decryptActionBoundArgs("60be8d67bb01f9fc1199f26e7d4bbc812d5446fe5337d20608749bb1de5f3e8caa", $$ACTION_CLOSURE_BOUND);
    $$ACTION_ARG_0(e);
    console.log(a, $$ACTION_ARG_1, $$ACTION_ARG_2, e);
};
registerServerReference($$RSC_SERVER_ACTION_1, "60be8d67bb01f9fc1199f26e7d4bbc812d5446fe5337d20608749bb1de5f3e8caa", null);
export const $$RSC_SERVER_ACTION_2 = async function action1($$ACTION_CLOSURE_BOUND, d) {
    var [$$ACTION_ARG_0, $$ACTION_ARG_1, $$ACTION_ARG_2] = await decryptActionBoundArgs("606bee4aba7e4600414a77a62a5fdf44cd90de849943a217961e7b080db8db5aa4", $$ACTION_CLOSURE_BOUND);
    let f;
    // @ts-expect-error: window is not iterable
    console.log(...window, {
        window
    });
    console.log(a, $$ACTION_ARG_0, action2);
    var action2 = $$RSC_SERVER_ACTION_0.bind(null, encryptActionBoundArgs("60d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", $$ACTION_ARG_1, d, f, $$ACTION_ARG_2));
    return [
        action2,
        $$RSC_SERVER_ACTION_1.bind(null, encryptActionBoundArgs("60be8d67bb01f9fc1199f26e7d4bbc812d5446fe5337d20608749bb1de5f3e8caa", action2, $$ACTION_ARG_1, d))
    ];
};
registerServerReference($$RSC_SERVER_ACTION_2, "606bee4aba7e4600414a77a62a5fdf44cd90de849943a217961e7b080db8db5aa4", null);
export async function action0(b, c, ...g) {
    return $$RSC_SERVER_ACTION_2.bind(null, encryptActionBoundArgs("606bee4aba7e4600414a77a62a5fdf44cd90de849943a217961e7b080db8db5aa4", b, c, g));
}
import { ensureServerActions } from "react-server-dom-rspack/server";
ensureServerActions([
    action0
]);
registerServerReference(action0, "7f6c187e7180343e58e03daff7bf02d6ff4613b7343de1bb40082b3ac2ecf5f677", null);
