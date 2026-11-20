import { loader } from "fumadocs-core/source";
import { lucideIconsPlugin } from "fumadocs-core/source/lucide-icons";
import { docsContentRoute, docsImageRoute, docsRoute } from "./shared";
import { defineCollections, defineDocs } from "fumadocs-mdx/macro";
import { metaSchema, pageSchema } from "fumadocs-core/source/schema";
import { blogPageSchema, type BlogAuthor } from "./blog";

const docs = defineDocs({
  dir: "content/docs",
  docs: {
    schema: pageSchema,
    postprocess: {
      includeProcessedMarkdown: true,
    },
  },
  meta: {
    schema: metaSchema,
  },
});

// See https://fumadocs.dev/docs/headless/source-api for more info
export const source = loader({
  baseUrl: docsRoute,
  source: docs.toFumadocsSource(),
  plugins: [lucideIconsPlugin()],
});

export function getPageImageUrl(page: (typeof source)["$inferPage"]) {
  const segments = [...page.slugs, "image.png"];

  return {
    segments,
    url:
      "/" +
      [page.locale, ...docsImageRoute.split("/"), ...segments]
        .filter(Boolean)
        .join("/"),
  };
}

export function getPageMarkdownUrl(page: (typeof source)["$inferPage"]) {
  const segments = [...page.slugs, "content.md"];

  return {
    segments,
    url:
      "/" +
      [page.locale, ...docsContentRoute.split("/"), ...segments]
        .filter(Boolean)
        .join("/"),
  };
}

export async function getLLMText(page: (typeof source)["$inferPage"]) {
  const processed = await page.data.getText("processed");

  return `# ${page.data.title} (${page.url})

${processed}`;
}

// BLOG

const blogPosts = defineCollections({
  type: "doc",
  dir: "content/blog",
  schema: blogPageSchema,
});

export const blogSource = loader({
  baseUrl: "/blog",
  source: blogPosts.toFumadocsSource(),
  plugins: [lucideIconsPlugin()],
});

export const blogAuthors: Record<string, BlogAuthor> = {
  "niklas-sauter": {
    name: "Niklas Sauter",
    title: "Maintainer",
    image: "https://avatars.githubusercontent.com/u/41170571",
  },
};
