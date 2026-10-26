"use client";

import type { NavNode } from "@/lib/nautilus-docs";
import { cn } from "@/lib/utils";
import Link from "next/link";
import { usePathname } from "next/navigation";

function containsPath(node: NavNode, pathname: string): boolean {
  return node.href === pathname || node.children.some((child) => containsPath(child, pathname));
}

function NavItem({ node, pathname, depth }: { node: NavNode; pathname: string; depth: number }) {
  const active = node.href === pathname;
  const link = node.href ? (
    <Link
      href={node.href}
      className={cn(
        "block rounded-md px-2 py-1 text-sm transition-colors hover:bg-muted",
        active ? "bg-muted font-medium text-foreground" : "text-muted-foreground",
        depth === 0 && "font-semibold text-foreground"
      )}
    >
      {node.title}
    </Link>
  ) : (
    <span className="block px-2 py-1 text-sm font-medium">{node.title}</span>
  );

  if (!node.children.length) return <li>{link}</li>;

  return (
    <li>
      <details open={depth === 0 || containsPath(node, pathname)} className="group">
        <summary className="flex cursor-pointer list-none items-center gap-1 [&::-webkit-details-marker]:hidden">
          <span className="text-xs text-muted-foreground transition-transform group-open:rotate-90">▶</span>
          <div className="flex-1">{link}</div>
        </summary>
        <ul className="ml-3 border-l pl-2">
          {node.children.map((child) => (
            <NavItem key={child.href ?? child.title} node={child} pathname={pathname} depth={depth + 1} />
          ))}
        </ul>
      </details>
    </li>
  );
}

export function DocsSidebar({ tree, home }: { tree: NavNode[]; home: string }) {
  const pathname = usePathname();
  return (
    <nav className="space-y-1">
      <Link
        href={home}
        className={cn(
          "block rounded-md px-2 py-1 text-sm font-semibold hover:bg-muted",
          pathname === home && "bg-muted"
        )}
      >
        หน้าแรก
      </Link>
      <ul className="space-y-1">
        {tree.map((node) => (
          <NavItem key={node.href ?? node.title} node={node} pathname={pathname} depth={0} />
        ))}
      </ul>
    </nav>
  );
}
