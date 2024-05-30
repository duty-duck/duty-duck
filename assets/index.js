import Alpine from 'alpinejs'
import ajax from '@imacrayon/alpine-ajax'

const feather = require("feather-icons/dist/feather");
require("bootstrap");

window.Alpine = Alpine
Alpine.plugin(ajax)
Alpine.start()

document.body.addEventListener("ajax:after", function () {
   feather.replace();
});
