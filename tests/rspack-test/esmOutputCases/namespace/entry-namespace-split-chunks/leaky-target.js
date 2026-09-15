import { sharedValue } from "./leaky-shared";

globalThis.entryNamespaceSplitOrder.push("leaky");
export const value = sharedValue;
