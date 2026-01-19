export function handleError(error: unknown): void {
  console.error("Global Error:", error);
  // Placeholder for UI notification (e.g., toast)
  // alert(error instanceof Error ? error.message : "An error occurred");
}
