import { registerServerReference } from "react-server-dom-rspack/server";
import { redirect } from 'navigation';
export const $$RSC_SERVER_ACTION_0 = async function action(formData) {
    redirect('/header?name=' + formData.get('name') + '&hidden-info=' + formData.get('hidden-info'));
};
registerServerReference($$RSC_SERVER_ACTION_0, "40d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", null);
var action = $$RSC_SERVER_ACTION_0;
export default function Form() {
    return /*#__PURE__*/ React.createElement("form", {
        action: action
    }, /*#__PURE__*/ React.createElement("input", {
        type: "text",
        name: "hidden-info",
        defaultValue: "hi",
        hidden: true
    }), /*#__PURE__*/ React.createElement("input", {
        type: "text",
        name: "name",
        id: "name",
        required: true
    }), /*#__PURE__*/ React.createElement("button", {
        type: "submit",
        id: "submit"
    }, "Submit"));
}
