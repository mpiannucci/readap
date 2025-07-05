import init, { 
  UrlBuilder, 
  parse_dds, 
  parse_das, 
  parse_dods,
  type DodsData
} from 'readap-wasm';

import type { 
  DAPClientOptions, 
  VariableConstraints, 
  FetchDataOptions,
  DatasetInfo,
  VariableData,
  DdsDataset,
  DasAttributes,
  VariableInfo,
  CoordinateInfo
} from './types.js';

export class DAPClient {
  private baseUrl: string;
  private options: DAPClientOptions;
  private urlBuilder?: UrlBuilder;
  private ddsCache?: DdsDataset;
  private dasCache?: DasAttributes;
  private initialized = false;

  constructor(baseUrl: string, options: DAPClientOptions = {}) {
    this.baseUrl = baseUrl.replace(/\/$/, ''); // Remove trailing slash
    this.options = {
      timeout: 30000,
      ...options
    };
  }

  async init(): Promise<void> {
    if (this.initialized) return;
    
    // In Node.js, we need to load the WASM file manually
    if (typeof window === 'undefined') {
      const fs = await import('fs');
      const path = await import('path');
      const { fileURLToPath } = await import('url');
      
      // Find the WASM file relative to this module
      const currentDir = path.dirname(fileURLToPath(import.meta.url));
      const wasmPath = path.resolve(currentDir, '../../readap-wasm/pkg/readap_wasm_bg.wasm');
      
      if (fs.existsSync(wasmPath)) {
        const wasmBytes = fs.readFileSync(wasmPath);
        await init(wasmBytes);
      } else {
        // Fallback to default init for web environments
        await init();
      }
    } else {
      // Web environment
      await init();
    }
    
    this.urlBuilder = new UrlBuilder(this.baseUrl);
    this.initialized = true;
  }

  private ensureInitialized(): void {
    if (!this.initialized || !this.urlBuilder) {
      throw new Error('DAPClient not initialized. Call init() first.');
    }
  }

  async getDatasetInfo(): Promise<DatasetInfo> {
    this.ensureInitialized();
    const [dds, das] = await Promise.all([
      this.getDDS(),
      this.getDAS()
    ]);

    // Extract variable and coordinate information from DDS
    const variables: VariableInfo[] = dds.variables.map((varName: string) => {
      const ddsValue = dds.values.find((v: any) => v.name === varName);
      if (!ddsValue) {
        throw new Error(`Variable ${varName} not found in DDS values`);
      }

      return this.extractVariableInfo(ddsValue);
    });

    const coordinates: CoordinateInfo[] = dds.coordinates.map((coordName: string) => {
      return this.extractCoordinateInfo(dds, coordName);
    });

    return {
      name: dds.name,
      variables,
      coordinates,
      attributes: das,
      dds
    };
  }

  async fetchData(variableName: string, options: FetchDataOptions = {}): Promise<VariableData> {
    this.ensureInitialized();
    const dds = await this.getDDS();
    
    // Verify variable exists
    if (!dds.variables.includes(variableName)) {
      throw new Error(`Variable '${variableName}' not found in dataset`);
    }

    // Build URL with constraints
    this.urlBuilder!.clearAll();
    this.urlBuilder!.addVariable(variableName);

    if (options.constraints) {
      this.applyConstraints(variableName, options.constraints);
    }

    // Fetch binary data
    const dodsUrl = this.urlBuilder!.dodsUrl();
    const response = await this.fetchWithTimeout(dodsUrl);
    
    if (!response.ok) {
      throw new Error(`Failed to fetch data: ${response.status} ${response.statusText}`);
    }

    const arrayBuffer = await response.arrayBuffer();
    const uint8Array = new Uint8Array(arrayBuffer);
    
    // Parse binary data using WASM
    const dodsDataset = parse_dods(uint8Array);
    
    // Extract the requested variable data and metadata
    const data = dodsDataset.getVariableData(variableName);
    const metadata = dodsDataset.getVariableInfo(variableName);
    
    // Clean up WASM memory
    dodsDataset.free();
    
    return {
      name: variableName,
      data,
      attributes: this.dasCache ? this.dasCache[variableName] : undefined,
      metadata
    };
  }

  private async getDDS(): Promise<DdsDataset> {
    if (this.ddsCache) {
      return this.ddsCache;
    }

    const ddsUrl = this.urlBuilder!.ddsUrl();
    const response = await this.fetchWithTimeout(ddsUrl);
    
    if (!response.ok) {
      throw new Error(`Failed to fetch DDS: ${response.status} ${response.statusText}`);
    }

    const ddsText = await response.text();
    this.ddsCache = parse_dds(ddsText) as DdsDataset;
    return this.ddsCache;
  }

  private async getDAS(): Promise<DasAttributes> {
    if (this.dasCache) {
      return this.dasCache;
    }

    const dasUrl = this.urlBuilder!.dasUrl();
    const response = await this.fetchWithTimeout(dasUrl);
    
    if (!response.ok) {
      throw new Error(`Failed to fetch DAS: ${response.status} ${response.statusText}`);
    }

    const dasText = await response.text();
    this.dasCache = parse_das(dasText) as DasAttributes;
    return this.dasCache;
  }

  private applyConstraints(variableName: string, constraints: VariableConstraints): void {
    for (const [coordName, constraint] of Object.entries(constraints)) {
      if (Array.isArray(constraint)) {
        if (constraint.length === 2) {
          // Range constraint [start, end]
          this.urlBuilder!.addRange(coordName, constraint[0], constraint[1]);
        } else {
          // Multiple indices
          for (const index of constraint) {
            this.urlBuilder!.addSingleIndex(coordName, index);
          }
        }
      } else if (typeof constraint === 'number') {
        // Single index
        this.urlBuilder!.addSingleIndex(coordName, constraint);
      }
    }
  }

  private async fetchWithTimeout(url: string): Promise<Response> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.options.timeout);

    try {
      const response = await fetch(url, {
        signal: controller.signal,
        headers: this.options.headers
      });
      return response;
    } finally {
      clearTimeout(timeoutId);
    }
  }

  private extractVariableInfo(ddsValue: any): VariableInfo {
    // This is a simplified extraction - in practice, we'd need to properly
    // map the DdsValue structure to VariableInfo
    return {
      name: ddsValue.name,
      dataType: ddsValue.dataType || 'String',
      variableType: ddsValue.type,
      coordinates: ddsValue.coordinates?.map((c: any) => c.name) || [],
      dimensions: ddsValue.coordinates || []
    };
  }

  private extractCoordinateInfo(dds: DdsDataset, coordName: string): CoordinateInfo {
    // Find coordinate information from the DDS
    // This is a simplified implementation
    const coordValue = dds.values.find((v: any) => v.name === coordName);
    if (!coordValue) {
      throw new Error(`Coordinate ${coordName} not found in DDS`);
    }

    return {
      name: coordName,
      dataType: (coordValue as any).dataType || 'String',
      size: (coordValue as any).coordinates?.[0]?.size || 0,
      variablesUsing: dds.variables.filter((varName: string) => {
        const varValue = dds.values.find((v: any) => v.name === varName);
        return (varValue as any).coordinates?.some((c: any) => c.name === coordName);
      })
    };
  }

  // Convenience methods for URL generation
  getDasUrl(): string {
    this.ensureInitialized();
    return this.urlBuilder!.dasUrl();
  }

  getDdsUrl(): string {
    this.ensureInitialized();
    return this.urlBuilder!.ddsUrl();
  }

  getDodsUrl(variables?: string[], constraints?: Record<string, any>): string {
    this.ensureInitialized();
    
    // Create a temporary builder to avoid affecting the main one
    let tempBuilder = new UrlBuilder(this.baseUrl);
    
    if (variables) {
      for (const variable of variables) {
        tempBuilder = tempBuilder.addVariable(variable);
      }
    }
    
    if (constraints) {
      for (const [varName, varConstraints] of Object.entries(constraints)) {
        if (Array.isArray(varConstraints)) {
          for (const constraint of varConstraints) {
            if (constraint.start !== undefined && constraint.end !== undefined) {
              if (constraint.stride !== undefined) {
                tempBuilder = tempBuilder.addRange(varName, constraint.start, constraint.end, constraint.stride);
              } else {
                tempBuilder = tempBuilder.addRange(varName, constraint.start, constraint.end);
              }
            }
          }
        }
      }
    }
    
    return tempBuilder.dodsUrl();
  }

  // Convenience methods for parsing without network calls
  parseDds(ddsContent: string): DdsDataset {
    this.ensureInitialized();
    return parse_dds(ddsContent) as DdsDataset;
  }

  parseDas(dasContent: string): DasAttributes {
    this.ensureInitialized();
    return parse_das(dasContent) as DasAttributes;
  }
}