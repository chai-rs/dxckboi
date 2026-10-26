import { MermaidRenderer } from "@/components/docs/mermaid";
import { DocsSidebar } from "@/components/docs/sidebar";
import { DOCS_BASE, getNavTree } from "@/lib/nautilus-docs";
import type { Metadata } from "next";
import { Noto_Sans_Thai } from "next/font/google";
import Link from "next/link";

const fontThai = Noto_Sans_Thai({
  subsets: ["thai", "latin"],
  variable: "--font-thai",
});

export const metadata: Metadata = {
  title: {
    default: "NautilusTrader Docs (ภาษาไทย)",
    template: "%s | NautilusTrader Docs (ภาษาไทย)",
  },
  description: "เอกสาร NautilusTrader ฉบับแปลภาษาไทย เน้น Rust",
};

export default function DocsLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const tree = getNavTree();
  return (
    <div className={`${fontThai.variable} docs-root mx-auto max-w-7xl px-4 pb-32 pt-8 sm:px-6`}>
      <header className="mb-8 flex flex-wrap items-baseline justify-between gap-2 border-b pb-4">
        <Link href={DOCS_BASE} className="text-lg font-semibold tracking-tight">
          NautilusTrader Docs <span className="text-muted-foreground">· ภาษาไทย (Rust)</span>
        </Link>
        <a
          href="https://nautilustrader.io/docs/latest/"
          className="text-sm text-muted-foreground hover:text-foreground"
          target="_blank"
          rel="noreferrer"
        >
          ต้นฉบับภาษาอังกฤษ ↗
        </a>
      </header>
      <div className="lg:grid lg:grid-cols-[16rem_minmax(0,1fr)] lg:gap-10">
        <aside className="mb-8 lg:mb-0">
          <details className="rounded-lg border p-3 lg:hidden">
            <summary className="cursor-pointer text-sm font-medium">สารบัญ</summary>
            <div className="mt-3">
              <DocsSidebar tree={tree} home={DOCS_BASE} />
            </div>
          </details>
          <div className="sticky top-8 hidden max-h-[calc(100vh-4rem)] overflow-y-auto pr-2 lg:block">
            <DocsSidebar tree={tree} home={DOCS_BASE} />
          </div>
        </aside>
        <div className="min-w-0">
          {children}
          <footer className="mt-16 border-t pt-6 text-xs text-muted-foreground">
            แปลจาก{" "}
            <a className="underline" href="https://github.com/nautechsystems/nautilus_trader/tree/develop/docs">
              เอกสาร NautilusTrader
            </a>{" "}
            © Nautech Systems Pty Ltd และผู้ร่วมพัฒนา เผยแพร่ภายใต้{" "}
            <a className="underline" href="https://www.gnu.org/licenses/lgpl-3.0.html">
              LGPL-3.0
            </a>{" "}
            ฉบับแปลนี้ตัดเนื้อหาเฉพาะ Python ออกและเน้น Rust
          </footer>
        </div>
      </div>
      <MermaidRenderer />
    </div>
  );
}
