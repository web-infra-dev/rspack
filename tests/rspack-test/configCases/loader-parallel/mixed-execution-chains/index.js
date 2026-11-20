it('should execute mixed main, parallel, and strict loader chains in order', async () => {
	const expected = [
		{
			phase: 'normal',
			name: 'main-6',
			parallel: false,
			strictBlocked: false,
			pitchData: {
				name: 'main-6',
				parallel: false,
				strictBlocked: false,
				remainingLen: 1,
				previousLen: 6,
			},
		},
		{
			phase: 'normal',
			name: 'parallel-5',
			parallel: true,
			strictBlocked: false,
			pitchData: {
				name: 'parallel-5',
				parallel: true,
				strictBlocked: false,
				remainingLen: 2,
				previousLen: 5,
			},
		},
		{
			phase: 'normal',
			name: 'strict-4',
			parallel: true,
			strictBlocked: true,
			pitchData: {
				name: 'strict-4',
				parallel: true,
				strictBlocked: true,
				remainingLen: 3,
				previousLen: 4,
			},
		},
		{
			phase: 'normal',
			name: 'strict-3',
			parallel: true,
			strictBlocked: true,
			pitchData: {
				name: 'strict-3',
				parallel: true,
				strictBlocked: true,
				remainingLen: 4,
				previousLen: 3,
			},
		},
		{
			phase: 'normal',
			name: 'parallel-nopitch-2',
			parallel: true,
			pitchData: null,
		},
		{
			phase: 'normal',
			name: 'parallel-nopitch-1',
			parallel: true,
			pitchData: null,
		},
		{
			phase: 'normal',
			name: 'main-0',
			parallel: false,
			strictBlocked: false,
			pitchData: {
				name: 'main-0',
				parallel: false,
				strictBlocked: false,
				remainingLen: 7,
				previousLen: 0,
			},
		},
	];
	const first = require('./mixed-all-modes-1');
	expect(first.entries).toEqual(expected);
	const second = await first.loadSecond();
	expect(second.default).toEqual(expected);
});

it('should short-circuit pitching across strict, parallel, and main chains', () => {
	expect(require('./pitch-shortcircuit')).toEqual([
		{
			phase: 'pitch-return',
			name: 'strict-2',
			parallel: true,
			strictBlocked: true,
		},
		{
			phase: 'normal',
			name: 'parallel-nopitch-1',
			parallel: true,
			pitchData: null,
		},
		{
			phase: 'normal',
			name: 'main-0',
			parallel: false,
			strictBlocked: false,
			pitchData: {
				name: 'main-0',
				parallel: false,
				strictBlocked: false,
				remainingLen: 5,
				previousLen: 0,
			},
		},
	]);
});

it('should serialize additionalData from main across native passthrough to parallel', () => {
	const result = require('./passthrough-additional-data');
	expect(result.parallel).toBe(true);
	expect(result.content.trim()).toBe('raw-content:from-main');
	expect(result.additionalData).toEqual({
		fromMain: 'ok',
		count: 42,
	});
});

it('should serialize additionalData from main pitch to parallel', () => {
	expect(require('./pitch-additional-data')).toEqual({
		parallel: true,
		content: 'from-main-pitch',
		additionalData: {
			fromPitch: true,
			count: 7,
		},
	});
});
