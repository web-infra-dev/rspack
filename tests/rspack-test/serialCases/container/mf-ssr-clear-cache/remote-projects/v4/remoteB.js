import { createRemoteExport } from "../createRemoteExport";

const remoteExport = createRemoteExport("./B", "v4");

export const payload = remoteExport.payload;
export default remoteExport.value;
