// Data types supported by OpenDAP
export type DataType = "Byte" | "Int16" | "UInt16" | "Int32" | "UInt32" | "Float32" | "Float64" | "String" | "URL";

// Variable types in DDS
export type VariableType = "Array" | "Grid" | "Structure" | "Sequence";

// Coordinate dimension information
export interface Dimension {
  name: string;
  size: number;
}

// DDS value structure (returned by WASM parse_dds)
export interface DdsValue {
  name: string;
  type: VariableType;
  dataType?: DataType;
  coordinates?: Dimension[];
  fields?: DdsValue[];
}

// Complete DDS dataset (returned by WASM parse_dds)
export interface DdsDataset {
  name: string;
  values: DdsValue[];
  variables: string[];
  coordinates: string[];
}

// DAS attribute structure
export interface DasAttribute {
  name: string;
  dataType: DataType;
  value: string;
}

// DAS attributes organized by variable (returned by WASM parse_das)
export interface DasAttributes {
  [variableName: string]: {
    [attributeName: string]: DasAttribute;
  };
}

// Variable metadata information
export interface VariableInfo {
  name: string;
  dataType: DataType;
  variableType: VariableType;
  coordinates: string[];
  dimensions: Dimension[];
}

// Coordinate metadata information
export interface CoordinateInfo {
  name: string;
  dataType: DataType;
  size: number;
  variablesUsing: string[];
}

export interface DAPClientOptions {
  timeout?: number;
  headers?: Record<string, string>;
}

export interface VariableConstraints {
  [coordinateName: string]: [number, number] | number | number[];
}

export interface FetchDataOptions {
  constraints?: VariableConstraints;
  format?: 'json' | 'array';
}

export interface DatasetInfo {
  name: string;
  variables: VariableInfo[];
  coordinates: CoordinateInfo[];
  attributes: DasAttributes;
  dds: DdsDataset;
}

export interface VariableData {
  name: string;
  data: Int8Array | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array | string[];
  attributes?: Record<string, any>;
  metadata: DdsValue;
}