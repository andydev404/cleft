import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";

export function CopyButton({ text }: { text: string }) {
  return (
    <Button size="sm" variant="outline" onClick={() => invoke("copy_to_clipboard", { text })}>
      Copy
    </Button>
  );
}
