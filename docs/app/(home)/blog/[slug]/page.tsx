import { notFound } from "next/navigation";
import { InlineTOC } from "fumadocs-ui/components/inline-toc";
import defaultMdxComponents from "fumadocs-ui/mdx";
import { blogSource } from "@/lib/source";

export default async function Page(props: {
  params: Promise<{ slug: string }>;
}) {
  const params = await props.params;
  const page = blogSource.getPage([params.slug]);

  if (!page) notFound();
  const Mdx = page.data.body;

  const formattedDate =
    page.data.date instanceof Date
      ? page.data.date.toISOString().slice(0, 10)
      : page.data.date;

  return (
    <>
      <div className="w-full max-w-200 mx-auto px-4 pt-8">
        <h1 className="mb-2 text-3xl">{page.data.title}</h1>
        <p className="mb-4 text-fd-muted-foreground">{formattedDate}</p>
      </div>
      <article className="w-full max-w-200 mx-auto flex flex-col px-4 py-4">
        <div className="prose min-w-0">
          <InlineTOC items={page.data.toc} />
          <Mdx components={defaultMdxComponents} />
        </div>
      </article>
    </>
  );
}

export function generateStaticParams(): { slug: string }[] {
  // TODO: remove once there is a blog post
  if (blogSource.getPages().length === 0) {
    return [{ slug: "__no_posts__" }];
  }

  return blogSource.getPages().map((page) => ({
    slug: page.slugs[0],
  }));
}

export async function generateMetadata(props: {
  params: Promise<{ slug: string }>;
}) {
  const params = await props.params;
  const page = blogSource.getPage([params.slug]);

  if (!page) notFound();

  return {
    title: page.data.title,
    date: page.data.date,
  };
}
