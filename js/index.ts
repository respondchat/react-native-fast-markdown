import fs from "fs";
import markdownit from "markdown-it";

const md = markdownit({
	linkify: true,
});
console.time("markdown-it");

for (let i = 0; i < 1000; i++) {
	const result = md.render(fs.readFileSync(__dirname + "/../TEST.md", "utf-8"));

	if (i === 0) {
		console.log(result);
	}
}

console.timeEnd("markdown-it");
