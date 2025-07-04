// Type definitions for readap-wasm

export interface DdsValue {
  name: string;
  type: "Array" | "Grid" | "Structure" | "Sequence";
  details: any;
}

export interface DdsDataset {
  name: string;
  values: DdsValue[];
  variables: string[];
  coordinates: string[];
}

export interface DasAttribute {
  name: string;
  type: "Byte" | "Int16" | "UInt16" | "Int32" | "UInt32" | "Float32" | "Float64" | "String" | "URL";
  value: any;
}

export interface DasAttributes {
  [variableName: string]: {
    [attributeName: string]: DasAttribute;
  };
}

export interface DodsDataset {
  dds: DdsDataset;
  variables: string[];
}

export interface VariableInfo {
  name: string;
  dataType: string;
  variableType: "Array" | "Grid" | "Structure" | "Sequence";
  coordinates: string[];
  dimensions: Array<{
    name: string;
    size: number;
  }>;
}

export interface CoordinateInfo {
  name: string;
  dataType: string;
  size: number;
  variablesUsing: string[];
}

export interface QueryBuilder {
  baseUrl: string;
  variables: string[];
  coordinates: string[];
}

export interface IndexRange {
  start?: number;
  end?: number;
  stride?: number;
}

export class JsUrlBuilder {
  constructor(baseUrl: string);
  
  dasUrl(): string;
  ddsUrl(): string;
  dodsUrl(): string;
  
  addVariable(variable: string): JsUrlBuilder;
  addVariables(variables: string[]): JsUrlBuilder;
  addSingleIndex(variable: string, index: number): JsUrlBuilder;
  addRange(variable: string, start: number, end: number): JsUrlBuilder;
  addRangeWithStride(variable: string, start: number, end: number, stride: number): JsUrlBuilder;
  addMultidimensionalConstraint(variable: string, indices: (number | IndexRange)[]): JsUrlBuilder;
  
  clearVariables(): JsUrlBuilder;
  clearConstraints(): JsUrlBuilder;
  clearAll(): JsUrlBuilder;
}

export function parse_dds(content: string): DdsDataset;
export function parse_das(content: string): DasAttributes;
export function parse_dods(bytes: Uint8Array): DodsDataset;

export function create_query_builder(ddsContent: string, baseUrl: string): QueryBuilder;
export function get_variable_info(ddsContent: string, variableName: string): VariableInfo;
export function get_coordinate_info(ddsContent: string, coordinateName: string): CoordinateInfo;