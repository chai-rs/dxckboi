import { Mermaid } from "@/components/docs/mermaid";
import defaultMdxComponents from "fumadocs-ui/mdx";
import type { MDXComponents } from "mdx/types";
import type { ComponentProps } from "react";

const DefaultImg = defaultMdxComponents.img;

// Remote images (status badges) skip the Next.js image optimizer, which rejects remote SVGs.
function Img(props: ComponentProps<typeof DefaultImg>) {
  if (typeof props.src === "string" && /^https?:\/\//.test(props.src)) {
    // eslint-disable-next-line @next/next/no-img-element
    return <img src={props.src} alt={props.alt ?? ""} style={{ display: "inline", margin: 0 }} />;
  }
  return <DefaultImg {...props} />;
}

export function getMDXComponents(components?: MDXComponents): MDXComponents {
  return {
    ...defaultMdxComponents,
    img: Img,
    Mermaid,
    ...components,
  };
}
