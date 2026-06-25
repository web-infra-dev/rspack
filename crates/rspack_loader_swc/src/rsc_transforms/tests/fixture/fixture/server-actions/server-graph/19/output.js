import { registerServerReference } from "react-server-dom-rspack/server";
import { encryptActionBoundArgs, decryptActionBoundArgs } from "react-server-dom-rspack/server";
import { Button } from 'components';
export const $$RSC_SERVER_ACTION_0 = async function action($$ACTION_CLOSURE_BOUND, value2) {
    var [$$ACTION_ARG_0] = await decryptActionBoundArgs("60d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", $$ACTION_CLOSURE_BOUND);
    return $$ACTION_ARG_0 * value2;
};
registerServerReference($$RSC_SERVER_ACTION_0, "60d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", null);
export function Item({ value }) {
    return /*#__PURE__*/ React.createElement(React.Fragment, null, /*#__PURE__*/ React.createElement(Button, {
        action: $$RSC_SERVER_ACTION_0.bind(null, encryptActionBoundArgs("60d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", value))
    }, "Multiple"));
}
