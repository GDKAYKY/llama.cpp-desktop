export interface Model {
    name: string;
    version: string;
    provider: string;
    library: string;
    full_identifier: string;
    manifest: {
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
    };
    model_file_path?: string;
}
