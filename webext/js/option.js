

(function () {

  function saveOptions(e) {
    e.preventDefault();
    browser.storage.local.set({
      apiEndPoint: document.querySelector("#api-end-point").value
    });
  }

  async function restoreOptions() {
    let values = await browser.storage.local.get();
    document.querySelector("#api-end-point").value = values.apiEndPoint || 'http://127.0.0.1:8116';
  }

  document.addEventListener("DOMContentLoaded", restoreOptions);
  document.querySelector("form").addEventListener("submit", saveOptions)
})();
