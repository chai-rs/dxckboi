import fs from "fs";
import matter from "gray-matter";
import path from "path";
import rehypePrettyCode from "rehype-pretty-code";
import rehypeStringify from "rehype-stringify";
import remarkDirective from "remark-directive";
import remarkGfm from "remark-gfm";
import remarkParse from "remark-parse";
import remarkRehype from "remark-rehype";
import { unified } from "unified";
import { visit } from "unist-util-visit";

export const DOCS_DIR = path.join(process.cwd(), "nautilus-docs-th");
export const DOCS_BASE = "/docs/nautilus";
export const ASSETS_BASE = `${DOCS_BASE}/assets`;
const UPSTREAM =
  "https://github.com/nautechsystems/nautilus_trader/blob/04f766b4d434d67fc8672026fefa25135bdc01be/docs";

const SECTION_ORDER = [
  "getting_started",
  "concepts",
  "how_to",
  "tutorials",
  "integrations",
  "developer_guide",
];

const ADMONITION_TITLES: Record<string, string> = {
  note: "หมายเหตุ",
  tip: "เคล็ดลับ",
  info: "ข้อมูล",
  warning: "คำเตือน",
  caution: "ข้อควรระวัง",
  danger: "อันตราย",
  important: "สำคัญ",
};

export type DocEntry = {
  // Path relative to DOCS_DIR, e.g. "concepts/orders/index.md"
  file: string;
  // URL slug segments, e.g. ["concepts", "orders"]
  slug: string[];
  title: string;
};

export type NavNode = {
  title: string;
  href?: string;
  children: NavNode[];
};

function fileToSlug(file: string): string[] {
  if (file === "README.md") return [];
  const parts = file.replace(/\.md$/, "").split("/");
  if (parts[parts.length - 1] === "index") parts.pop();
  return parts;
}

export function slugToHref(slug: string[]): string {
  return slug.length ? `${DOCS_BASE}/${slug.join("/")}` : DOCS_BASE;
}

function readTitle(file: string): string {
  const { content } = matter(fs.readFileSync(path.join(DOCS_DIR, file), "utf-8"));
  const h1 = content.match(/^#\s+(.+)$/m);
  return h1 ? h1[1].trim() : path.basename(file, ".md");
}

function walk(dir: string): string[] {
  return fs.readdirSync(path.join(DOCS_DIR, dir), { withFileTypes: true }).flatMap((entry) => {
    const rel = dir ? `${dir}/${entry.name}` : entry.name;
    if (entry.isDirectory()) return walk(rel);
    return entry.name.endsWith(".md") ? [rel] : [];
  });
}

let entriesCache: DocEntry[] | undefined;

export function getAllDocs(): DocEntry[] {
  entriesCache ??= walk("").map((file) => ({
    file,
    slug: fileToSlug(file),
    title: readTitle(file),
  }));
  return entriesCache;
}

export function getDocBySlug(slug: string[]): DocEntry | undefined {
  const key = slug.join("/");
  return getAllDocs().find((doc) => doc.slug.join("/") === key);
}

export function getNavTree(): NavNode[] {
  const docs = getAllDocs();
  const byFile = new Map(docs.map((doc) => [doc.file, doc]));
  const dirs = new Set(docs.map((doc) => path.posix.dirname(doc.file)));

  function build(dir: string): NavNode {
    const index = byFile.get(`${dir}/index.md`);
    const pages = docs
      .filter((doc) => path.posix.dirname(doc.file) === dir && !doc.file.endsWith("/index.md"))
      .sort((a, b) => a.file.localeCompare(b.file))
      .map((doc) => ({ title: doc.title, href: slugToHref(doc.slug), children: [] }));
    const subdirs = Array.from(dirs)
      .filter((d) => path.posix.dirname(d) === dir)
      .sort()
      .map(build);
    return {
      title: index?.title ?? path.basename(dir),
      href: index ? slugToHref(index.slug) : undefined,
      children: [...pages, ...subdirs],
    };
  }

  return SECTION_ORDER.filter((section) => byFile.has(`${section}/index.md`)).map(build);
}

// Resolve a relative markdown link from `fromFile` to a site URL.
function resolveLink(url: string, fromFile: string): string {
  if (/^([a-z]+:|#|\/)/i.test(url)) return url;
  const [target, anchor] = url.split("#");
  const hash = anchor ? `#${anchor}` : "";
  const resolved = path.posix.normalize(path.posix.join(path.posix.dirname(fromFile), target));
  const candidates = [
    resolved,
    `${resolved}.md`,
    `${resolved.replace(/\/$/, "")}/index.md`,
  ];
  const docs = getAllDocs();
  for (const candidate of candidates) {
    const doc = docs.find((d) => d.file === candidate);
    if (doc) return slugToHref(doc.slug) + hash;
  }
  // Non-markdown files (e.g. Rust templates) link to the upstream repository.
  return `${UPSTREAM}/${resolved}${hash}`;
}

// Docusaurus `:::type Title` -> remark-directive `:::type[Title]`, outside code fences.
function normalizeAdmonitions(markdown: string): string {
  let inFence = false;
  return markdown
    .split("\n")
    .map((line) => {
      if (/^\s*```/.test(line)) inFence = !inFence;
      if (inFence) return line;
      const m = line.match(/^(\s*):::(\w+)\s+(.+)$/);
      return m && !m[3].startsWith("[") ? `${m[1]}:::${m[2]}[${m[3].trim()}]` : line;
    })
    .join("\n");
}

function remarkNautilus(file: string) {
  return (tree: any) => {
    visit(tree, (node: any, index, parent: any) => {
      // `## Heading {#anchor}` -> <h2 id="anchor">
      if (node.type === "heading") {
        const last = node.children[node.children.length - 1];
        const m = last?.type === "text" && last.value.match(/\s*\{#([^}]+)\}\s*$/);
        if (m) {
          last.value = last.value.slice(0, m.index);
          node.data = { ...node.data, hProperties: { id: m[1] } };
        }
      }

      if (node.type === "containerDirective") {
        const first = node.children[0];
        const hasLabel = first?.data?.directiveLabel;
        const title = hasLabel ? first.children : [{ type: "text", value: ADMONITION_TITLES[node.name] ?? node.name }];
        if (hasLabel) node.children.shift();
        node.children.unshift({
          type: "paragraph",
          data: { hProperties: { className: ["admonition-title"] } },
          children: title,
        });
        node.data = {
          hName: "div",
          hProperties: { className: ["admonition", `admonition-${node.name}`] },
        };
      }

      // Only container admonitions are intended; restore accidental `:word` / `::word` text.
      if ((node.type === "textDirective" || node.type === "leafDirective") && parent && index !== undefined) {
        const prefix = node.type === "textDirective" ? ":" : "::";
        const restored = [{ type: "text", value: `${prefix}${node.name}` }, ...node.children];
        parent.children.splice(
          index,
          1,
          ...(node.type === "leafDirective" ? [{ type: "paragraph", children: restored }] : restored)
        );
        return index;
      }

      if (node.type === "code" && node.lang === "mermaid") {
        node.data = {
          hName: "div",
          hProperties: { className: ["mermaid"], "data-code": node.value },
          hChildren: [{ type: "text", value: node.value }],
        };
      }

      if (node.type === "link") node.url = resolveLink(node.url, file);
      if (node.type === "image" && !/^[a-z]+:/i.test(node.url)) {
        node.url = `${ASSETS_BASE}/${path.posix.normalize(path.posix.join(path.posix.dirname(file), node.url))}`;
      }
    });
  };
}

function rehypeWrapTables() {
  return (tree: any) => {
    visit(tree, "element", (node: any, index, parent: any) => {
      if (node.tagName !== "table" || !parent || index === undefined) return;
      parent.children[index] = {
        type: "element",
        tagName: "div",
        properties: { className: ["table-wrapper"] },
        children: [node],
      };
    });
  };
}

export async function renderDoc(doc: DocEntry): Promise<string> {
  const raw = fs.readFileSync(path.join(DOCS_DIR, doc.file), "utf-8");
  const { content } = matter(raw);
  const result = await unified()
    .use(remarkParse)
    .use(remarkGfm)
    .use(remarkDirective)
    .use(remarkNautilus, doc.file)
    .use(remarkRehype, { allowDangerousHtml: true })
    .use(rehypePrettyCode, {
      theme: { light: "min-light", dark: "min-dark" },
      keepBackground: false,
    })
    .use(rehypeWrapTables)
    .use(rehypeStringify, { allowDangerousHtml: true })
    .process(normalizeAdmonitions(content));
  return result.toString();
}
