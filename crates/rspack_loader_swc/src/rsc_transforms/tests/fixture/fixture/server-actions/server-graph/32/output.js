import { registerServerReference } from "react-server-dom-rspack/server";
import { encryptActionBoundArgs, decryptActionBoundArgs } from "react-server-dom-rspack/server";
export const $$RSC_SERVER_ACTION_0 = async function action($$ACTION_CLOSURE_BOUND) {
    var [$$ACTION_ARG_0, $$ACTION_ARG_1, $$ACTION_ARG_2] = await decryptActionBoundArgs("40d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", $$ACTION_CLOSURE_BOUND);
    console.log($$ACTION_ARG_0.at(1), $$ACTION_ARG_1, $$ACTION_ARG_1.current);
    console.log($$ACTION_ARG_2.push.call($$ACTION_ARG_2, 5));
};
registerServerReference($$RSC_SERVER_ACTION_0, "40d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", null);
export function Component() {
    const data = [
        1,
        2,
        3
    ];
    const foo = [
        2,
        3,
        4
    ];
    const baz = {
        value: {
            current: 1
        }
    };
    var action = $$RSC_SERVER_ACTION_0.bind(null, encryptActionBoundArgs("40d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", data, baz.value, foo));
    return /*#__PURE__*/ React.createElement("form", {
        action: action
    });
}
