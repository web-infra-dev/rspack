import { registerServerReference } from "react-server-dom-rspack/server";
import { validator } from 'auth';
import { Button } from 'components';
export const $$RSC_SERVER_ACTION_0 = async function myAction(a, b, c) {
    console.log('a');
};
registerServerReference($$RSC_SERVER_ACTION_0, "70d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", null);
var myAction = $$RSC_SERVER_ACTION_0;
export default function Page() {
    return /*#__PURE__*/ React.createElement(Button, {
        action: myAction
    }, "Delete");
}
export const $$RSC_SERVER_ACTION_1 = async function() {};
registerServerReference($$RSC_SERVER_ACTION_1, "00be8d67bb01f9fc1199f26e7d4bbc812d5446fe5337d20608749bb1de5f3e8caa", null);
// TODO: should use `action` as function name?
export const action = validator($$RSC_SERVER_ACTION_1);
