import fs from "fs";
import path from "path";
import { visit } from "unist-util-visit";

const DOCS_DIR = path.join(process.cwd(), "nautilus-docs-th");
export const DOCS_BASE = "/docs/nautilus";
const UPSTREAM =
  "https://github.com/nautechsystems/nautilus_trader/blob/04f766b4d434d67fc8672026fefa25135bdc01be/docs";

// Turn a relative markdown link into a site URL; non-doc files point to the upstream repository.
function resolveLink(url: string, fromFile: string): string {
  if (/^([a-z]+:|#|\/)/i.test(url)) return url;
  const [target, anchor] = url.split("#");
  const hash = anchor ? `#${anchor}` : "";
  const resolved = path.posix.normalize(path.posix.join(path.posix.dirname(fromFile), target)).replace(/\/$/, "");
  for (const candidate of [resolved, `${resolved}.md`, `${resolved}/index.md`]) {
    if (!candidate.endsWith(".md") || !fs.existsSync(path.join(DOCS_DIR, candidate))) continue;
    const slug = candidate.replace(/\.md$/, "").replace(/(^|\/)index$/, "");
    return (slug ? `${DOCS_BASE}/${slug}` : DOCS_BASE) + hash;
  }
  return `${UPSTREAM}/${resolved}${hash}`;
}

/**
 * Adapts the translated NautilusTrader docs to Fumadocs:
 * - only `:::` container directives are intended, so stray `:word` / `::word` text is restored
 * - `<br>` in table cells becomes a line break; other raw HTML is shown as text
 * - relative `.md` links resolve to `/docs/nautilus/...`
 */
export function remarkNautilus() {
  return (tree: any, file: { path?: string }) => {
    const fromFile = file.path ? path.relative(DOCS_DIR, file.path).split(path.sep).join("/") : "";

    visit(tree, (node: any, index, parent: any) => {
      if (!parent || index === undefined) return;

      if (node.type === "textDirective" || node.type === "leafDirective") {
        const prefix = node.type === "textDirective" ? ":" : "::";
        const restored = [{ type: "text", value: `${prefix}${node.name}` }, ...node.children];
        parent.children.splice(
          index,
          1,
          ...(node.type === "leafDirective" ? [{ type: "paragraph", children: restored }] : restored)
        );
        return index;
      }

      if (node.type === "html") {
        parent.children[index] = /^<br\s*\/?>$/i.test(node.value.trim())
          ? { type: "break" }
          : { type: "text", value: node.value };
      }

      if (node.type === "link") node.url = resolveLink(node.url, fromFile);
    });
  };
}
