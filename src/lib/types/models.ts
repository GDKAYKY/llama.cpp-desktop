export interface Model {
    name: string;
    version: string;
    provider: string;
    library: string;
    full_identifier: string;
    manifest_data: ModelManifest;
    model_file_path?: string;
    manifest_path?: string;
}

export interface ModelManifest {
    schema_version: number;
    media_type: string;
    config: {
        mediaType: string;
        digest: string;
        size: number;
    };
    layers: Array<{
        mediaType: string;
        digest: string;
        size: number;
    }>;
}
