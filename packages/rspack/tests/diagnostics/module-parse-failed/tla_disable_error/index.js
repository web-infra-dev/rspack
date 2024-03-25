const a = await Promise.resolve("aaa");

for await (const _ of [Promise.resolve("bbbb")]) { }

export default a;
