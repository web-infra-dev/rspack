import { createRemoteExport } from "../createRemoteExport";

const remoteExport = createRemoteExport("./A", "v3");

export const payload = remoteExport.payload;
export default remoteExport.value;
