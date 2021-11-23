const postcss = require("postcss");

const file = require("fs").readFileSync("./assets/bootstrap.css", "utf8");


console.time("label");
const output = postcss().use(require("postcss-aspect-ratio-mini")).process(file).css;
console.timeEnd("label");
