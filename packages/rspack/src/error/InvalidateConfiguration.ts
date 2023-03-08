export default class InvalidateConfigurationError extends Error {
	constructor(message: string) {
		super(message);
		this.name = "InvalidateConfigurationError";
	}
}
