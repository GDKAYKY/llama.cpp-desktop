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
    tokenizer_metadata?: Record<string, unknown>;
    model_file_path?: string;
}
