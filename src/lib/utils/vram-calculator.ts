/**
 * VRAM Usage Calculator
 * Estimates GPU memory requirements based on model and inference parameters
 */

/**
 * Estimates VRAM usage in MB based on model configuration
 *
 * @param modelSizeBytes - Total model size in bytes
 * @param gpuLayers - Number of layers offloaded to GPU (-1 for all)
 * @param contextSize - Context window size in tokens
 * @param parallelSlots - Number of parallel inference slots
 * @param totalLayers - Total number of layers in the model (optional, for -1 calculation)
 * @returns Estimated VRAM usage in MB
 */
export function estimateVramUsage(
  modelSizeBytes: number,
  gpuLayers: number,
  contextSize: number,
  parallelSlots: number = 1,
  totalLayers: number = 32,
): number {
  if (modelSizeBytes <= 0) return 0;

  // Calculate model weights on GPU
  let modelWeightsVram = 0;
  if (gpuLayers > 0) {
    // Approximate: each layer is roughly equal size
    const layerSize = modelSizeBytes / totalLayers;
    modelWeightsVram = (layerSize * gpuLayers) / (1024 * 1024); // Convert to MB
  } else if (gpuLayers === -1) {
    // All layers on GPU
    modelWeightsVram = modelSizeBytes / (1024 * 1024);
  }

  // KV cache estimation
  // Typical: ~2 bytes per token per layer (fp16)
  // For a 7B model: ~32 layers, hidden size ~4096
  const kvCachePerSlot = (contextSize * 32 * 4096 * 2) / (1024 * 1024); // MB per slot
  const kvCacheVram = kvCachePerSlot * parallelSlots;

  // Temporary buffers and overhead (~10-15% of model weights)
  const overheadVram = modelWeightsVram * 0.12;

  return Math.round(modelWeightsVram + kvCacheVram + overheadVram);
}

/**
 * Formats VRAM usage for display
 * @param vramMB - VRAM usage in MB
 * @returns Formatted string (e.g., "4.2 GB" or "512 MB")
 */
export function formatVramUsage(vramMB: number): string {
  if (vramMB >= 1024) {
    return `${(vramMB / 1024).toFixed(1)} GB`;
  }
  return `${Math.round(vramMB)} MB`;
}

/**
 * Gets a human-readable description of VRAM usage level
 * @param vramMB - VRAM usage in MB
 * @returns Description string
 */
export function getVramDescription(vramMB: number): string {
  if (vramMB < 1024) {
    return "Very light - suitable for most GPUs";
  } else if (vramMB < 4096) {
    return "Light - 4GB+ GPU recommended";
  } else if (vramMB < 8192) {
    return "Moderate - 8GB+ GPU recommended";
  } else if (vramMB < 16384) {
    return "Heavy - 16GB+ GPU recommended";
  } else if (vramMB < 24576) {
    return "Very heavy - 24GB+ GPU recommended";
  } else {
    return "Extreme - 48GB+ GPU recommended";
  }
}

/**
 * Gets a color indicator for VRAM usage level
 * @param vramMB - VRAM usage in MB
 * @returns Tailwind color class
 */
export function getVramColorClass(vramMB: number): string {
  if (vramMB < 1024) {
    return "text-green-400";
  } else if (vramMB < 4096) {
    return "text-blue-400";
  } else if (vramMB < 8192) {
    return "text-yellow-400";
  } else if (vramMB < 16384) {
    return "text-orange-400";
  } else {
    return "text-red-400";
  }
}
