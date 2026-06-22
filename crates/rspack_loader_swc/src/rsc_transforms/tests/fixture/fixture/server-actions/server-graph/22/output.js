import { registerServerReference } from "react-server-dom-rspack/server";
import { validator } from 'auth';
export const action = validator(async ()=>{});
const $$RSC_SERVER_ACTION_0 = validator(async ()=>{});
export default $$RSC_SERVER_ACTION_0;
import { ensureServerActions } from "react-server-dom-rspack/server";
ensureServerActions([
    $$RSC_SERVER_ACTION_0,
    action
]);
registerServerReference($$RSC_SERVER_ACTION_0, "7f629c080094d3e42e3f04da3fc764e18c5905c42019036be89980b7d901dc746d", null);
registerServerReference(action, "7fa2596154c7aa07f03b5ef8eaef000c8894b51876f19ae00d947fa104c0d5857e", null);
