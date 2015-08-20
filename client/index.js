// main.js

var pageMod = require("sdk/page-mod");

pageMod.PageMod({
  include: "*.chat.stackoverflow.com",
  contentScriptFile: ["./jquery.js", "./polyfill.js", "./background.js"],
  contentStyleFile: "./style.css",
});
