let sdk;

export async function initOpenTelemetry() {
	if (sdk) return;
	const otel = await import("@opentelemetry/sdk-node");
	const { Resource } = await import("@opentelemetry/resources");
	const { OTLPTraceExporter } = await import(
		"@opentelemetry/exporter-trace-otlp-proto"
	);
	const { AsyncHooksContextManager } = await import(
		"@opentelemetry/context-async-hooks"
	);
	const contextManager = new AsyncHooksContextManager();
	contextManager.enable();
	otel.api.context.setGlobalContextManager(contextManager);

	sdk = new otel.NodeSDK({
		resource: new Resource({
			"service.name": "rspack-app"
		}),
		traceExporter: new OTLPTraceExporter()
	});
	sdk.start();
}

export async function shutdownOpenTelemetry() {
	if (!sdk) return;
	const otel = await import("@opentelemetry/sdk-node");
	await sdk.shutdown();
	otel.api.context.disable();
	sdk = null;
}

export { trace, propagation, context } from "@opentelemetry/api";
