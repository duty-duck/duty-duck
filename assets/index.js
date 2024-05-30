import Alpine from "alpinejs";
import AlpineAjax from "@imacrayon/alpine-ajax";
import "./location.store";
import IntervalInput from "./interval-input";

const feather = require("feather-icons");
require("bootstrap");

window.Alpine = Alpine;
Alpine.plugin(AlpineAjax);
Alpine.data("intervalInput", IntervalInput);

const load = () => {
  feather.replace();
  Alpine.store("location").update();
};

document.body.addEventListener("ajax:merged", load);

load();
Alpine.start();
