import { toast } from "svelte-sonner";

/**
 * Copy text to clipboard with toast notification
 */
export async function copyToClipboard(
  text,
  successMessage = "Copied to clipboard",
  errorMessage = "Failed to copy to clipboard",
) {
  try {
    if (navigator.clipboard && navigator.clipboard.writeText) {
      await navigator.clipboard.writeText(text);
      toast.success(successMessage);
      return true;
    }

    // Fallback for non-secure contexts
    const textArea = document.createElement("textarea");
    textArea.value = text;
    textArea.style.position = "fixed";
    textArea.style.left = "-9999px";
    textArea.style.top = "-9999px";
    document.body.appendChild(textArea);
    textArea.focus();
    textArea.select();

    const successful = document.execCommand("copy");
    document.body.removeChild(textArea);

    if (successful) {
      toast.success(successMessage);
      return true;
    } else {
      throw new Error("execCommand failed");
    }
  } catch (error) {
    console.error("Failed to copy to clipboard:", error);
    toast.error(errorMessage);
    return false;
  }
}

export async function copyCodeToClipboard(
  rawCode,
  successMessage = "Code copied to clipboard",
  errorMessage = "Failed to copy code",
) {
  return copyToClipboard(rawCode, successMessage, errorMessage);
}
