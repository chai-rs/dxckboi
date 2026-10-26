import { DOCS_DIR } from "@/lib/nautilus-docs";
import fs from "fs";
import path from "path";

export const dynamic = "force-static";
export const dynamicParams = false;

const CONTENT_TYPES: Record<string, string> = {
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".jpeg": "image/jpeg",
  ".gif": "image/gif",
  ".svg": "image/svg+xml",
  ".webp": "image/webp",
};

function listImages(dir: string): string[] {
  return fs.readdirSync(path.join(DOCS_DIR, dir), { withFileTypes: true }).flatMap((entry) => {
    const rel = dir ? `${dir}/${entry.name}` : entry.name;
    if (entry.isDirectory()) return listImages(rel);
    return CONTENT_TYPES[path.extname(entry.name).toLowerCase()] ? [rel] : [];
  });
}

export function generateStaticParams() {
  return listImages("").map((file) => ({ path: file.split("/") }));
}

export function GET(_request: Request, { params }: { params: { path: string[] } }) {
  const file = path.join(DOCS_DIR, ...params.path);
  const contentType = CONTENT_TYPES[path.extname(file).toLowerCase()];
  if (!contentType || !file.startsWith(DOCS_DIR + path.sep) || !fs.existsSync(file)) {
    return new Response("Not found", { status: 404 });
  }
  return new Response(fs.readFileSync(file), {
    headers: { "Content-Type": contentType, "Cache-Control": "public, max-age=31536000, immutable" },
  });
}
