export interface TeraTemplateAnalysis {
	variables: string[]
	variableCount: number
	hasLoops: boolean
	hasConditionals: boolean
	loopCount: number
	conditionalCount: number
	iterableVariables: string[]
	sampleValues: Record<string, string[]>
	defaultFallbacks: Record<string, string>
	filterUsage: Record<string, string[]>
}

export interface TemplateExcelPreview {
	sheetNames: string[]
	selectedSheet: string
	headerRowIndex: number
	columns: string[]
	previewRows: string[][]
	totalRows: number
	columnsWithData: string[]
	invalidIterableColumns: string[]
}

export interface GenericGeneratedConfig {
	label: string
	config: string
	rowIndex: number
}
