const path = __non_webpack_require__("path");

const getHashPrefix = (value, local) => value.slice(0, -`-${local}`.length);
const getSnapshotPath = name => {
  const suffix = typeof document === "undefined" ? "-node" : "";
  return path.join(__SNAPSHOT__, name.replace(/\.txt$/, `${suffix}.txt`));
};

it("should have correct local ident for css export locals", async () => {
  const [
    idLocal,
    hash,
    hashLocal,
    fullhashLocal,
    pathNameLocal,
    fileLocal,
    queryFragment,
    uniqueNameIdContenthash,
    hashLocalCustom,
    less
  ] = await Promise.all([
    import("./style.module.css"),
    import("./style.module.css?hash"),
    import("./style.module.css?hash-local"),
    import("./style.module.css?fullhash-local"),
    import("./style.module.css?path-name-local"),
    import("./style.module.css?file-local"),
    import("./style.module.css?q#f"),
    import("./style.module.css?uniqueName-id-contenthash"),
    import("./style.module.css?hash-local-custom"),
    import("./style.module.less")
  ]);

  expect(getHashPrefix(hashLocal.simple, "simple")).toBe(
    getHashPrefix(hashLocal.foo_bar, "foo_bar")
  );
  expect(getHashPrefix(hashLocal.simple, "simple")).toBe(
    getHashPrefix(hashLocal["btn-info_is-disabled"], "btn-info_is-disabled")
  );
  expect(getHashPrefix(hashLocal.simple, "simple")).toBe(
    getHashPrefix(hashLocal["btn--info_is-disabled_1"], "btn--info_is-disabled_1")
  );
  expect(getHashPrefix(hashLocalCustom.simple, "simple")).toBe(
    getHashPrefix(hashLocalCustom.foo_bar, "foo_bar")
  );
  expect(getHashPrefix(fullhashLocal.simple, "simple")).not.toBe(
    getHashPrefix(fullhashLocal.foo_bar, "foo_bar")
  );

  expect(idLocal).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, "id-local.txt"));
  expect(hash).toMatchFileSnapshotSync(getSnapshotPath("hash.txt"));
  expect(hashLocal).toMatchFileSnapshotSync(getSnapshotPath("hash-local.txt"));
  expect(fullhashLocal).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, "fullhash-local.txt"));
  expect(pathNameLocal).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, "path-name-local.txt"));
  expect(fileLocal).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, "file-local.txt"));
  expect(queryFragment).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, "query-fragment.txt"));
  expect(uniqueNameIdContenthash).toMatchFileSnapshotSync(
    path.join(__SNAPSHOT__, "unique-name-id-contenthash.txt")
  );
  expect(hashLocalCustom).toMatchFileSnapshotSync(getSnapshotPath("hash-local-custom.txt"));
  expect(less).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, "less.txt"));
});
