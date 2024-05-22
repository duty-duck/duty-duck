const feather = require("feather-icons/dist/feather");
require("bootstrap");
window.htmx = require("htmx.org");
require("htmx.org/dist/ext/head-support");
require("htmx.org/dist/ext/preload");

document.body.addEventListener("htmx:load", function (evt) {
  feather.replace();
});
