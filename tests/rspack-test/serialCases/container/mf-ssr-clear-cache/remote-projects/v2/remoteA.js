import { createRemoteExport } from "../createRemoteExport";

const remoteExport = createRemoteExport("./A", "v2");

export const payload = remoteExport.payload;
export default remoteExport.value;
