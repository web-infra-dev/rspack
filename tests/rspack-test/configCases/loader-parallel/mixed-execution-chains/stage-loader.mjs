export function pitch(remainingRequest, previousRequest, data) {
	const { name, expectStrict, pitchReturn } = this.getOptions();
	let strictBlocked = false;
	if (expectStrict) {
		try {
			void this._compiler;
		} catch {
			strictBlocked = true;
		}
	}
	data.pitch = {
		name,
		parallel: Boolean(this.parallel),
		strictBlocked,
		remainingLen: remainingRequest ? remainingRequest.split('!').length : 0,
		previousLen: previousRequest ? previousRequest.split('!').length : 0,
	};
	if (pitchReturn !== undefined) {
		return JSON.stringify([
			{
				phase: 'pitch-return',
				name,
				parallel: Boolean(this.parallel),
				strictBlocked,
			},
		]);
	}
}

export default function (source) {
	const { name, expectStrict, emitResult } = this.getOptions();
	let strictBlocked = false;
	if (expectStrict) {
		try {
			void this._compiler;
		} catch {
			strictBlocked = true;
		}
	}
	const entries =
		source && source.trim().startsWith('[') ? JSON.parse(source) : [];
	entries.push({
		phase: 'normal',
		name,
		parallel: Boolean(this.parallel),
		strictBlocked,
		pitchData: this.data?.pitch ?? null,
	});
	if (emitResult) {
		if (this.resourcePath.endsWith('mixed-all-modes-1.js')) {
			return `module.exports = { entries: ${JSON.stringify(entries)}, loadSecond: () => import('./mixed-all-modes-2.js') };`;
		}
		return `module.exports = ${JSON.stringify(entries)};`;
	}
	return JSON.stringify(entries);
}
