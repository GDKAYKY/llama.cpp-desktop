import { invokeCommand } from '../infrastructure/ipc';
import type { Model } from '../types/models';

export async function downloadModelFromRegistry(modelsRoot: string, modelReference: string): Promise<Model> {
  return await invokeCommand('download_model_from_registry', { modelsRoot, modelReference }) as Promise<Model>;
}
