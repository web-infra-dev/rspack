$RUNTIME_HANDLERS$ = {};
$RUNTIME_FUNCTION$ = function (chunkId) {
  Object.keys($RUNTIME_HANDLERS$).map(function (key) {
    $RUNTIME_HANDLERS$[key](chunkId);
  });
}
