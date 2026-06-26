import { createRemoteExport } from "../createRemoteExport";

const remoteExport = createRemoteExport("./A", "v1");

export const payload = remoteExport.payload;
export default remoteExport.value;
