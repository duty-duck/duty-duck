import Alpine from "alpinejs";

Alpine.store("location", {
  path: "",
  update() {
    this.path = window.location.pathname;
  },
  matchesHref(el) {
    return el.getAttribute("href") == this.path;
  },
  startsWithHref(el) {
    return this.path.startsWith(el.getAttribute("href"));
  },
});
