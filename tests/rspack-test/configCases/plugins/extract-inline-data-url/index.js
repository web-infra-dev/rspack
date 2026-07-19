const direct = 'data:image/png;base64,aGVsbG8gaW5saW5lIGFzc2V0';
const embedded = JSON.parse(
  '{"src":"data:image/png;base64,aGVsbG8gaW5saW5lIGFzc2V0"}',
);
globalThis.inlineDataUrls = [direct, embedded.src];
