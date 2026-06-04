import { registerServerReference } from "react-server-dom-rspack/server";
import { encryptActionBoundArgs, decryptActionBoundArgs } from "react-server-dom-rspack/server";
import { validator, another } from 'auth';
import { Button } from 'components';
const x = 1;
export const $$RSC_SERVER_ACTION_0 = async function($$ACTION_CLOSURE_BOUND, z) {
    var [$$ACTION_ARG_0] = await decryptActionBoundArgs("60d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", $$ACTION_CLOSURE_BOUND);
    return x + $$ACTION_ARG_0 + z;
};
registerServerReference($$RSC_SERVER_ACTION_0, "60d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", null);
export default function Page() {
    const y = 1;
    return /*#__PURE__*/ React.createElement(Button, {
        // TODO: should use `action` as function name?
        action: validator($$RSC_SERVER_ACTION_0.bind(null, encryptActionBoundArgs("60d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", y)))
    });
}
export const $$RSC_SERVER_ACTION_1 = async function() {};
registerServerReference($$RSC_SERVER_ACTION_1, "00be8d67bb01f9fc1199f26e7d4bbc812d5446fe5337d20608749bb1de5f3e8caa", null);
validator($$RSC_SERVER_ACTION_1);
export const $$RSC_SERVER_ACTION_2 = async function() {};
registerServerReference($$RSC_SERVER_ACTION_2, "006bee4aba7e4600414a77a62a5fdf44cd90de849943a217961e7b080db8db5aa4", null);
another(validator($$RSC_SERVER_ACTION_2));
