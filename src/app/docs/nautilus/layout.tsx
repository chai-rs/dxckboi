import { source } from "@/lib/source";
import { DocsLayout } from "fumadocs-ui/layouts/docs";
import { RootProvider } from "fumadocs-ui/provider/next";
import type { Metadata } from "next";
import { Noto_Sans_Thai } from "next/font/google";
import type { ReactNode } from "react";

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

const translations = {
  "Search(search trigger)": "ค้นหา",
  "Search(search dialog)": "ค้นหาเอกสาร",
  "No results found(search dialog)": "ไม่พบผลลัพธ์",
  "Search failed(search dialog)": "ค้นหาไม่สำเร็จ",
  "On this page(table of contents)": "ในหน้านี้",
  "No Headings(table of contents)": "ไม่มีหัวข้อ",
  "Table of Contents(inline table of contents)": "สารบัญ",
  "Next Page(pagination)": "หน้าถัดไป",
  "Previous Page(pagination)": "หน้าก่อนหน้า",
  "Last updated on(page footer)": "อัปเดตล่าสุดเมื่อ",
  "Page Not Found(404 page)": "ไม่พบหน้านี้",
  "Back to Home(404 page)": "กลับหน้าแรก",
  "Show Sidebar(sidebar)": "แสดงเมนู",
  "Hide Sidebar(sidebar)": "ซ่อนเมนู",
};

export default function Layout({ children }: { children: ReactNode }) {
  return (
    <div className={`${fontThai.variable} docs-root`}>
      <RootProvider theme={{ enabled: false }} i18n={{ locale: "th", translations }}>
        <DocsLayout
          tree={source.getPageTree()}
          nav={{ title: "NautilusTrader Docs · ภาษาไทย" }}
          links={[
            { text: "หน้าแรกของเว็บ", url: "/" },
            { text: "ต้นฉบับภาษาอังกฤษ", url: "https://nautilustrader.io/docs/latest/", external: true },
          ]}
        >
          {children}
        </DocsLayout>
      </RootProvider>
    </div>
  );
}
