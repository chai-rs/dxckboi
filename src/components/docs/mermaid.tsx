"use client";

import { useTheme } from "next-themes";
import { usePathname } from "next/navigation";
import { useEffect } from "react";

// Renders every `.mermaid` block on the page; re-renders on navigation and theme change.
export function MermaidRenderer() {
  const pathname = usePathname();
  const { resolvedTheme } = useTheme();

  useEffect(() => {
    const blocks = Array.from(document.querySelectorAll<HTMLElement>(".mermaid[data-code]"));
    if (!blocks.length) return;
    let cancelled = false;

    import("mermaid").then(({ default: mermaid }) => {
      if (cancelled) return;
      for (const block of blocks) {
        block.removeAttribute("data-processed");
        block.textContent = block.dataset.code ?? "";
      }
      mermaid.initialize({
        startOnLoad: false,
        theme: resolvedTheme === "dark" ? "dark" : "default",
        securityLevel: "strict",
      });
      mermaid.run({ nodes: blocks, suppressErrors: true });
    });

    return () => {
      cancelled = true;
    };
  }, [pathname, resolvedTheme]);

  return null;
}
