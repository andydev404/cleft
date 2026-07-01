import { Button } from "@/components/ui/button";

interface PermissionRunwayProps {
  onGrant: () => void;
  onContinueWithout: () => void;
}

// Content and framing follow the Cleft-PRD.md Permission Runway spec: the
// system dialog's alarming wording ("wants to control your computer") is
// explained before it appears, not after — that's the single most
// important piece of copy in the app.
export function PermissionRunway({ onGrant, onContinueWithout }: PermissionRunwayProps) {
  return (
    <main className="w-screen h-screen flex flex-col items-center justify-center gap-6 p-8 text-center">
      <h1 className="text-lg font-semibold">
        Cleft needs one permission to work properly.
      </h1>

      <div className="w-full max-w-md rounded-lg border p-4 text-left text-sm space-y-3">
        <div>
          <div className="font-medium">Accessibility</div>
        </div>
        <div>
          <span className="text-muted-foreground">Reads: </span>
          Active app name + window title, browser URL (via accessibility
          tree), active file in your code editor
        </div>
        <div>
          <span className="text-muted-foreground">Never: </span>
          Screen content, keystrokes, passwords, mouse position
        </div>
        <div>
          <span className="text-muted-foreground">Stored: </span>
          Encrypted. 100% local. Always.
        </div>
      </div>

      <p className="max-w-md text-sm text-muted-foreground">
        This is what lets you find "that SQL I ran in TablePlus yesterday"
        instead of scrolling through hundreds of raw clips.
      </p>

      <Button onClick={onGrant}>Grant Accessibility Permission</Button>

      <p className="max-w-md text-xs text-muted-foreground">
        Note: macOS will show a system dialog next. It will say "Cleft wants
        to control your computer." This is Apple's standard wording for
        Accessibility. It does not mean Cleft controls your mouse or
        keyboard. Click Allow.
      </p>

      <button
        onClick={onContinueWithout}
        className="text-xs text-muted-foreground underline underline-offset-2"
      >
        Continue without this
      </button>
    </main>
  );
}
