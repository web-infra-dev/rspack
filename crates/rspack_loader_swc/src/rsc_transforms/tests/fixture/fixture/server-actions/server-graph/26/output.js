import { registerServerReference } from "react-server-dom-rspack/server";
const noop = (action)=>action;
export const $$RSC_SERVER_ACTION_0 = async function(data) {
    console.log(data);
};
registerServerReference($$RSC_SERVER_ACTION_0, "40d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", null);
// TODO: should use `log` as function name?
export const log = noop($$RSC_SERVER_ACTION_0);
