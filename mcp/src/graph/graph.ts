import { Record } from "neo4j-driver";
import { db } from "./neo4j.js";
import archy from "archy";
import { buildTree } from "./codemap.js";
import { code_body } from "./codebody.js";
import { Express, Request, Response } from "express";
import { upload_files, check_status } from "./uploads.js";

export function graph_routes(app: Express) {
  app.get("/pages", get_pages);
  app.get("/pages/links", get_pages_links);
  app.get("/feature_map", get_feature_map);
  app.get("/feature_code", get_feature_code);
  app.get("/components/links", get_components_links);
  app.post("/upload", upload_files);
  app.get("/status/:requestId", check_status);
}

function toPage(rec: Record): any {
  const page = rec.get("page");
  return {
    node_type: page.labels[0],
    ...page.properties,
  };
}

async function get_pages(req: Request, res: Response) {
  try {
    const result = await db.get_pages();
    const pages = result.records.map(toPage);
    res.json(pages);
  } catch (error) {
    console.error("Error:", error);
    res.status(500).send("Internal Server Error");
  }
}

interface Params {
  page_name: string | null;
  function_name: string | null;
  tests: boolean;
}

function params(req: Request): Params {
  const page_name = req.query.page_name as string;
  const function_name = req.query.function_name as string;
  const tests = !(req.query.tests === "false" || req.query.tests === "0");
  if (!page_name && !function_name)
    throw new Error("page or function required");
  return {
    page_name: page_name || null,
    function_name: function_name || null,
    tests,
  };
}

async function get_feature_map(req: Request, res: Response) {
  try {
    const { page_name, function_name, tests } = params(req);
    console.log("=> get_feature_map:", page_name, function_name, tests);
    const result = await db.get_function_path(page_name, function_name, tests);
    const fn = result.records[0];
    const tree = buildTree(fn);
    const text = archy(tree);
    res.send(`<pre>${text}</pre>`);
  } catch (error) {
    console.error("Error:", error);
    res.status(500).send("Internal Server Error");
  }
}

async function get_feature_code(req: Request, res: Response) {
  try {
    const { page_name, function_name, tests } = params(req);
    const pkg_files = await db.get_pkg_files();
    const result = await db.get_function_path(page_name, function_name, tests);
    const text = code_body(result.records[0], pkg_files);
    res.send(text);
  } catch (error) {
    console.error("Error:", error);
    res.status(500).send("Internal Server Error");
  }
}

async function get_pages_links(req: Request, res: Response) {
  try {
    const result = await db.get_pages();
    const pages = result.records.map(toPage);
    const html = createLinksList(pages, "page");
    res.send(html);
  } catch (error) {
    console.error("Error:", error);
    res.status(500).send("Internal Server Error");
  }
}

function createLinksList(data: any[], type: "page" | "function") {
  let html = "<ul>";

  data.forEach((item) => {
    function makehref(map_or_code: "map" | "code") {
      return `/feature_${map_or_code}?${type}_name=${encodeURIComponent(
        item.name
      )}`;
    }
    html += `<li>
      <strong style="width: 280px; display: inline-block; text-align: right;">${
        item.name
      }</strong>
      <a href="${makehref("map")}">[map]</a>
      <a href="${makehref("code")}">[code]</a>
      <span>(${item.file})</span>
    </li>`;
  });

  html += "</ul>";
  return html;
}

function toComponent(rec: Record): any {
  const page = rec.get("component");
  return {
    node_type: page.labels[0],
    ...page.properties,
  };
}

async function get_components_links(req: Request, res: Response) {
  try {
    const result = await db.get_components();
    const components = result.records.map(toComponent);
    const html = createLinksList(components, "function");
    res.send(html);
  } catch (error) {
    console.error("Error:", error);
    res.status(500).send("Internal Server Error");
  }
}
