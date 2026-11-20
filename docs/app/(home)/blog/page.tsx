import { blogAuthors, blogSource } from "@/lib/source";
import { getPosts } from "@/lib/blog";
import { BlogCards } from "@/components/blog";

export default function Page() {
  return (
    <div className="mx-auto w-full max-w-[900px] flex flex-col gap-6 px-4 pt-6 pb-12">
      <div>
        <h1 className="text-2xl font-semibold">Lokey Blog</h1>
        <p className="text-fd-muted-foreground mt-1">
          Latest announcements of Lokey
        </p>
      </div>
      <BlogCards posts={getPosts(blogSource.getPages(), blogAuthors)} />
    </div>
  );
}
