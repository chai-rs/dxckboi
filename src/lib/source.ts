import { docs } from "collections/server";
import { loader } from "fumadocs-core/source";
import { DOCS_BASE } from "./remark-nautilus";

export const source = loader({
  baseUrl: DOCS_BASE,
  source: docs.toFumadocsSource(),
});
