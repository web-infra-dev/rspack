export function initOpenTelemetry(): Promise<void>;
export function shutdownOpenTelemetry(): Promise<void>;
export { trace, propagation, context } from "@opentelemetry/api";
