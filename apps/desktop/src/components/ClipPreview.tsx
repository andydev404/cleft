import type { ContentType } from "@/types";
import { CodePreview } from "@/components/previews/CodePreview";
import { ColorPreview } from "@/components/previews/ColorPreview";
import { EmailPreview } from "@/components/previews/EmailPreview";
import { FilePathPreview } from "@/components/previews/FilePathPreview";
import { JsonPreview } from "@/components/previews/JsonPreview";
import { MarkdownPreview } from "@/components/previews/MarkdownPreview";
import { PlainTextPreview } from "@/components/previews/PlainTextPreview";
import { UrlPreview } from "@/components/previews/UrlPreview";

export function ClipPreview({ contentType, content }: { contentType: ContentType; content: string | null }) {
  if (content === null) {
    // Content arrives from a single indexed SQLite read — rendering nothing
    // instead of a "Loading…" placeholder avoids a flash of DOM swap.
    return null;
  }

  switch (contentType) {
    case "Color":
      return <ColorPreview value={content} />;
    case "JSON":
      return <JsonPreview value={content} />;
    case "Markdown":
      return <MarkdownPreview value={content} />;
    case "Code":
    case "SQL":
      return <CodePreview value={content} label={contentType} />;
    case "Email":
      return <EmailPreview value={content} />;
    case "FilePath":
      return <FilePathPreview value={content} />;
    case "URL":
      return <UrlPreview value={content} />;
    default:
      return <PlainTextPreview value={content} />;
  }
}
