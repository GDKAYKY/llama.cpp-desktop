export interface Model {
    name: string;
    version: string;
    provider: string;
    library: string;
    full_identifier: string;
    manifest: {
        layers: Array<{
            size: number;
        }>;
    };
    model_file_path?: string;
}
