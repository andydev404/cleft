import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export function MarkdownPreview({ value }: { value: string }) {
  const [html, setHtml] = useState<string | null>(null);

  useEffect(() => {
    invoke<string>("render_markdown", { content: value }).then(setHtml);
  }, [value]);

  if (html === null) return null;
  // Safe to inject: render_markdown sanitizes with ammonia in Rust before
  // this ever reaches the frontend — see src-tauri/src/markdown.rs.
  return (
    <div
      className="text-[13px] leading-[1.6] [&_a]:underline [&_code]:rounded [&_code]:bg-muted [&_code]:px-1 [&_h1]:text-lg [&_h1]:font-semibold [&_h2]:font-semibold [&_ul]:list-disc [&_ul]:pl-4"
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}
