import type { DAPClientOptions, FetchDataOptions, DatasetInfo, VariableData, DdsDataset, DasAttributes } from './types.js';
export declare class DAPClient {
    private baseUrl;
    private options;
    private urlBuilder?;
    private ddsCache?;
    private dasCache?;
    private initialized;
    constructor(baseUrl: string, options?: DAPClientOptions);
    init(): Promise<void>;
    private ensureInitialized;
    getDatasetInfo(): Promise<DatasetInfo>;
    fetchData(variableName: string, options?: FetchDataOptions): Promise<VariableData>;
    private getDDS;
    private getDAS;
    private applyConstraints;
    private fetchWithTimeout;
    private extractVariableInfo;
    private extractCoordinateInfo;
    getDasUrl(): string;
    getDdsUrl(): string;
    getDodsUrl(variables?: string[], constraints?: Record<string, any>): string;
    parseDds(ddsContent: string): DdsDataset;
    parseDas(dasContent: string): DasAttributes;
}
//# sourceMappingURL=client.d.ts.map