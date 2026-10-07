import Link from "next/link";
import { getBlogPosts } from "@/lib/source";

export default function Home() {
  const posts = getBlogPosts();

  return (
    <main className="flex-1 w-full max-w-200 mx-auto px-4 py-8">
      <h1 className="text-3xl mb-8">Latest Blog Posts</h1>
      <div className="flex flex-col divide-y">
        {posts.map((post) => (
          <Link
            key={post.url}
            href={post.url}
            className="block py-4 transition-colors hover:underline"
          >
            <h2 className="text-xl font-semibold">{post.data.title}</h2>
            <p className="mt-0.5 text-fd-muted-foreground">
              {post.data.date instanceof Date
                ? post.data.date.toISOString().slice(0, 10)
                : post.data.date}
            </p>
          </Link>
        ))}
      </div>
    </main>
  );
}
