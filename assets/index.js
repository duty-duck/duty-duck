import Alpine from "alpinejs";
import ajax from "@imacrayon/alpine-ajax";
const feather = require("feather-icons");
require("bootstrap");

window.Alpine = Alpine;
Alpine.plugin(ajax);

const load = () => {
  feather.replace();
  Alpine.store("location", {
    path: window.location.pathname,
    matchesHref(el) {
      return el.getAttribute("href") == this.path;
    }
  });
};

document.body.addEventListener("ajax:after", load);
load();

Alpine.start();