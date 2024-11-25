const { resolve } = require("path");
const express = require("express");
const vhost = require("vhost");
const serveStatic = require("serve-static");

function getRandomInt(min, max) {
  const minCeiled = Math.ceil(min);
  const maxFloored = Math.floor(max);
  return Math.floor(Math.random() * (maxFloored - minCeiled) + minCeiled);
}

// A middleare to add a latency to some websites
const delayMiddlware = delay => (req, res, next) => {
  setTimeout(next, delay)
}

const staticFolder = resolve(__dirname, "static");

const mediapartApp = express();

mediapartApp.use(delayMiddlware(500))
mediapartApp.use(serveStatic(resolve(staticFolder, "mediapart")));

const fakeInternetApp = express();
fakeInternetApp.get("/slow-endpoint", delayMiddlware(3500), (req, res) => {
  res.send("Hello there!")
});
fakeInternetApp.get("/very-slow-endpoint", delayMiddlware(12000), (req, res) => {
  res.send("Hello there!")
});
fakeInternetApp.get("/status/:status", (req, res) => {
  res.status(Number(req.params.status)).send("A response with a special HTTP status")
});
// A flaky endpoint that a 1 chance out of n of failing
fakeInternetApp.get("/flaky/:n", (req, res) => {
  let fail = getRandomInt(0, Number(req.params.n)) == 0;
  if (fail) {
    res.status(500).send("ERROR!");
  } else {
    res.send("OK")
  }
})

const mainApp = express();
mainApp.use(vhost("www.mediapart.fr", mediapartApp));
mainApp.use(vhost("www.fake-internet.com", fakeInternetApp));

const port = process.env.PORT ? Number(process.env.PORT) : 3001;
console.log("Starting fake internet on port", port);
mainApp.listen(port, "0.0.0.0", () => {
  console.log("Fake internet is listening on port", port);
});
