import { invokeCommand } from "../infrastructure/ipc";

export async function removeModelByIdentifier(
  fullIdentifier: string,
  modelsRoot: string,
): Promise<boolean> {
  return (await invokeCommand("remove_model_by_identifier", {
    fullIdentifier,
    modelsRoot,
  })) as boolean;
}

export async function removeModelByManifestPath(
  manifestPath: string,
  modelsRoot: string,
): Promise<boolean> {
  return (await invokeCommand("remove_model_by_manifest_path", {
    manifestPath,
    modelsRoot,
  })) as boolean;
}
