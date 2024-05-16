(async () => {
  import("../../pkg/apex_client")
    .then(wasm => wasm.init())
    .catch(console.error);
})();
