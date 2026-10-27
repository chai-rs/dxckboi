import { remarkDirectiveAdmonition, remarkMdxMermaid } from "fumadocs-core/mdx-plugins";
import { defineConfig, defineDocs } from "fumadocs-mdx/config";
import remarkDirective from "remark-directive";
import { remarkNautilus } from "./src/lib/remark-nautilus";

export const docs = defineDocs({
  dir: "nautilus-docs-th",
  docs: {
    // README.md is for browsing the folder on GitHub; index.md is the site's landing page.
    files: ["**/*.md", "!README.md"],
  },
});

export default defineConfig({
  mdxOptions: {
    // Don't fetch remote images (status badges) at build time to measure them.
    remarkImageOptions: { external: false },
    remarkPlugins: (v) => [
      remarkDirective,
      remarkNautilus,
      [
        remarkDirectiveAdmonition,
        {
          types: {
            note: "info",
            tip: "success",
            info: "info",
            important: "info",
            warning: "warning",
            caution: "warning",
            danger: "error",
          },
        },
      ],
      remarkMdxMermaid,
      ...v,
    ],
  },
});
