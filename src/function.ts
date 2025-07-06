import { createNodeMiddleware, createProbot } from "probot";
import app from "./app.js";

exports.probotApp = createNodeMiddleware(app, {
	probot: createProbot(),
	webhooksPath: "/",
});
