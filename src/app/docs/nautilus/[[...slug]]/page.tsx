import { getMDXComponents } from "@/components/docs/mdx-components";
import { source } from "@/lib/source";
import { DocsBody, DocsDescription, DocsPage, DocsTitle } from "fumadocs-ui/layouts/docs/page";
import type { Metadata } from "next";
import { notFound } from "next/navigation";

type Props = { params: Promise<{ slug?: string[] }> };

export const dynamicParams = false;

export function generateStaticParams() {
  return source.generateParams();
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const page = source.getPage((await params).slug);
  if (!page) return {};
  return { title: page.data.title, description: page.data.description };
}

export default async function Page({ params }: Props) {
  const page = source.getPage((await params).slug);
  if (!page) notFound();
  const MDX = page.data.body;

  return (
    <DocsPage toc={page.data.toc} full={page.data.full}>
      <DocsTitle>{page.data.title}</DocsTitle>
      {page.data.description && <DocsDescription>{page.data.description}</DocsDescription>}
      <DocsBody>
        <MDX components={getMDXComponents()} />
      </DocsBody>
      <p className="mt-8 border-t pt-4 text-xs text-fd-muted-foreground">
        แปลจาก{" "}
        <a className="underline" href="https://github.com/nautechsystems/nautilus_trader/tree/develop/docs">
          เอกสาร NautilusTrader
        </a>{" "}
        © Nautech Systems Pty Ltd และผู้ร่วมพัฒนา เผยแพร่ภายใต้{" "}
        <a className="underline" href="https://www.gnu.org/licenses/lgpl-3.0.html">
          LGPL-3.0
        </a>{" "}
        ฉบับแปลนี้ตัดเนื้อหาเฉพาะ Python ออกและเน้น Rust
      </p>
    </DocsPage>
  );
}
