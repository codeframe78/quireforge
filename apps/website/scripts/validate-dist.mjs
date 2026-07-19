import { readFile, readdir } from "node:fs/promises";
import { extname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = resolve(fileURLToPath(new URL("..", import.meta.url)));
const distRoot = join(packageRoot, "dist");
const origin = "https://quireforge.jamesjennison.net";

const requiredFiles = [
  "index.html",
  "404.html",
  "_headers",
  "robots.txt",
  "site.webmanifest",
  "sitemap-index.xml",
  "features/index.html",
  "integrations/index.html",
  "downloads/index.html",
  "installation/index.html",
  "documentation/index.html",
  "compatibility/index.html",
  "roadmap/index.html",
  "releases/index.html",
  "security/index.html",
  "contributing/index.html",
  "faq/index.html",
  "troubleshooting/index.html",
  "about/index.html",
];

const errors = [];

async function exists(path) {
  try {
    await readFile(path);
    return true;
  } catch {
    return false;
  }
}

async function walk(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) files.push(...(await walk(path)));
    else files.push(path);
  }
  return files;
}

function resolveAsset(htmlFile, rawTarget) {
  const target = rawTarget.split("#", 1)[0].split("?", 1)[0];
  if (!target || target.startsWith("#")) return null;
  if (/^(?:https?:|mailto:|tel:|data:)/.test(target)) return null;

  const decoded = decodeURIComponent(target);
  const base = decoded.startsWith("/") ? distRoot : resolve(htmlFile, "..");
  const candidate = decoded.startsWith("/")
    ? join(base, decoded.slice(1))
    : resolve(base, decoded);

  if (decoded.endsWith("/")) return join(candidate, "index.html");
  if (!extname(candidate)) return join(candidate, "index.html");
  return candidate;
}

for (const required of requiredFiles) {
  if (!(await exists(join(distRoot, required)))) {
    errors.push(`missing generated file: ${required}`);
  }
}

const allFiles = await walk(distRoot);
const htmlFiles = allFiles.filter((path) => path.endsWith(".html"));

for (const htmlFile of htmlFiles) {
  const name = relative(distRoot, htmlFile);
  const html = await readFile(htmlFile, "utf8");

  if (!html.includes('<main id="main-content">')) {
    errors.push(`missing main landmark: ${name}`);
  }
  if (!html.includes("QuireForge is an unofficial community project")) {
    errors.push(`missing unofficial-project disclaimer: ${name}`);
  }
  if (!html.includes(`<link rel="canonical" href="${origin}`)) {
    errors.push(`missing production canonical URL: ${name}`);
  }
  if (/<style(?:\s|>)/i.test(html) || /\sstyle=/i.test(html)) {
    errors.push(`inline style conflicts with the production CSP: ${name}`);
  }
  for (const script of html.matchAll(/<script\b([^>]*)>/gi)) {
    if (!/\ssrc=/i.test(script[1])) {
      errors.push(`inline script conflicts with the production CSP: ${name}`);
    }
  }

  for (const match of html.matchAll(/\s(?:href|src)="([^"]+)"/gi)) {
    const target = resolveAsset(htmlFile, match[1]);
    if (target && !(await exists(target))) {
      errors.push(`broken generated link in ${name}: ${match[1]}`);
    }
  }
}

const headers = await readFile(join(distRoot, "_headers"), "utf8");
for (const directive of [
  "Content-Security-Policy:",
  "Permissions-Policy:",
  "Referrer-Policy:",
  "X-Content-Type-Options:",
]) {
  if (!headers.includes(directive))
    errors.push(`missing security header: ${directive}`);
}
if (headers.includes("Strict-Transport-Security")) {
  errors.push(
    "HSTS must remain disabled until the production hostname is verified",
  );
}

if (errors.length > 0) {
  console.error("Generated website validation failed:");
  for (const error of errors) console.error(`- ${error}`);
  process.exitCode = 1;
} else {
  console.log(
    `Generated website validation passed (${htmlFiles.length} HTML pages).`,
  );
}
