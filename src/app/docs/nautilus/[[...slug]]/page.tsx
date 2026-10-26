import { getAllDocs, getDocBySlug, renderDoc } from "@/lib/nautilus-docs";
import type { Metadata } from "next";
import { notFound } from "next/navigation";

export const dynamicParams = false;

type Props = { params: { slug?: string[] } };

export function generateStaticParams() {
  return getAllDocs().map((doc) => ({ slug: doc.slug }));
}

export function generateMetadata({ params }: Props): Metadata {
  const doc = getDocBySlug(params.slug ?? []);
  return doc ? { title: doc.title } : {};
}

export default async function DocPage({ params }: Props) {
  const doc = getDocBySlug(params.slug ?? []);
  if (!doc) notFound();
  const html = await renderDoc(doc);
  return <article className="docs-prose prose max-w-none dark:prose-invert" dangerouslySetInnerHTML={{ __html: html }} />;
}
