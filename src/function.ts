import type { IncomingMessage, ServerResponse } from "node:http";
import type { HttpFunction } from "@google-cloud/functions-framework";
import { createNodeMiddleware, createProbot } from "probot";
import app from "./app.js";

const probot = createProbot();
const middleware = createNodeMiddleware(app, {
	probot,
	webhooksPath: "/",
});

export const probotApp: HttpFunction = async (
	req: IncomingMessage,
	res: ServerResponse,
) => {
	middleware(req, res);
};
