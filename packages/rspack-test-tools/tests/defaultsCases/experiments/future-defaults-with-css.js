module.exports = {
	description: "experiments.futureDefaults w/ experiments.css disabled",
	options: () => ({
		experiments: {
			css: false,
			futureDefaults: true
		}
	}),
	diff: e =>
		e.toMatchInlineSnapshot(`
    - Expected
    + Received

    @@ ... @@
    -     "css": true,
    +     "css": false,
    +     "futureDefaults": true,
    @@ ... @@
    -       },
    -       Object {
    -         "oneOf": Array [
    -           Object {
    -             "resolve": Object {
    -               "fullySpecified": true,
    -             },
    -             "test": /\\.module\\.css$/i,
    -             "type": "css/module",
    @@ ... @@
    -             "resolve": Object {
    -               "fullySpecified": true,
    -               "preferRelative": true,
    -             },
    -             "type": "css",
    -           },
    -         ],
    -         "test": /\\.css$/i,
    -       },
    -       Object {
    -         "mimetype": "text/css+module",
    -         "resolve": Object {
    -           "fullySpecified": true,
    -         },
    -         "type": "css/module",
    -       },
    -       Object {
    -         "mimetype": "text/css",
    -         "resolve": Object {
    -           "fullySpecified": true,
    -           "preferRelative": true,
    -         },
    -         "type": "css",
    -       },
    -       Object {
    @@ ... @@
    -     "hashDigestLength": 20,
    -     "hashFunction": "md4",
    +     "hashDigestLength": 16,
    +     "hashFunction": "xxhash64",
  `)
};
