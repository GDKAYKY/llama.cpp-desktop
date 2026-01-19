import { clsx } from "clsx";
import { twMerge } from "tailwind-merge";

/**
 * Combine tailwind classes with conditional logic and merging.
 * @param {...string} inputs - Class names or conditional class objects
 * @returns {string} Combined and merged class names
 */
export function cn(...inputs) {
  return twMerge(clsx(inputs));
}
