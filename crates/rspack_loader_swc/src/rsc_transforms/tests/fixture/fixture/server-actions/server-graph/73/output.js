import { registerServerReference } from "react-server-dom-rspack/server";
import { validator } from 'auth';
export const action = validator(async ()=>{});
import { ensureServerActions } from "react-server-dom-rspack/server";
ensureServerActions([
    action
]);
registerServerReference(action, "7fa2596154c7aa07f03b5ef8eaef000c8894b51876f19ae00d947fa104c0d5857e", null);
