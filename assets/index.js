const feather = require("feather-icons/dist/feather");
require("bootstrap");
window.htmx = require("htmx.org");
require("htmx.org/dist/ext/head-support");
require("htmx.org/dist/ext/preload");
require("htmx.org/dist/ext/loading-states");

document.body.addEventListener("htmx:load", function (evt) {
  feather.replace();
});

// Workaround to change body class using hx-boost
// See https://github.com/bigskysoftware/htmx/issues/1384
document.body.addEventListener("htmx:afterSwap", function (evt) {
  if (evt.target.tagName == "BODY") {
    const parser = new DOMParser();
    const parsedResponse = parser.parseFromString(
      evt.detail.xhr.response,
      "text/html"
    );
    const bodyAttributes =
      parsedResponse.getElementsByTagName("body")[0].attributes;
    for (const attribute of bodyAttributes) {
      evt.detail.target.setAttribute(attribute.name, attribute.value);
    }
  }
});
