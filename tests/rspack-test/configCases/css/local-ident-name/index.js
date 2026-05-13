const path = __non_webpack_require__("path");

it("should have correct local ident for css export locals", async () => {
  const [idLocal, hash, hashLocal, pathNameLocal, fileLocal, queryFragment, uniqueNameIdContenthash, less] = await Promise.all([
    import("./style.module.css"),
    import("./style.module.css?hash"),
    import("./style.module.css?hash-local"),
    import("./style.module.css?path-name-local"),
    import("./style.module.css?file-local"),
    import("./style.module.css?q#f"),
    import("./style.module.css?uniqueName-id-contenthash"),
    import("./style.module.css?hash-local-custom"),
    import("./style.module.less"),
  ]);

  expect(idLocal).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, "id-local.txt"));
  expect(hash).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, "hash.txt"));
  expect(hashLocal).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, "hash-local.txt"));
  expect(pathNameLocal).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, "path-name-local.txt"));
  expect(fileLocal).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, "file-local.txt"));
  expect(queryFragment).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, "query-fragment.txt"));
  expect(uniqueNameIdContenthash).toMatchFileSnapshotSync(
    path.join(__SNAPSHOT__, "unique-name-id-contenthash.txt")
  );
  expect(less).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, "less.txt"));
});
