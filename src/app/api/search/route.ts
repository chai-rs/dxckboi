import { source } from "@/lib/source";
import { createFromSource } from "fumadocs-core/search/server";

// Searched on the server: the full index (mostly Thai text) is too large to send to the browser.
// The default tokenizer segments Thai words via Intl.Segmenter.
export const { GET } = createFromSource(source);
